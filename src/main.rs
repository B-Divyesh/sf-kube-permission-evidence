use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use kube_permission_evidence::{AccessMatrix, EvidenceReport, Snapshot, evaluate, parse_subject};
use kube_permission_evidence::{collector, packet, render};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "kpe",
    version,
    about = "Explain effective Kubernetes RBAC access as audit evidence"
)]
#[command(
    long_about = "Kube Permission Evidence collects RBAC objects read-only, evaluates a bounded access matrix locally, and writes every causal grant to Markdown and JSON. It never changes cluster state or retains kubeconfig tokens."
)]
#[command(
    after_help = "Start safely:\n  kpe snapshot --output rbac-snapshot.json --context audit-readonly\n  kpe report --snapshot rbac-snapshot.json --subject user:alice --matrix matrix.json --output evidence\n\nSubject forms: user:<name> | group:<name> | serviceaccount:<namespace>:<name>"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Collect a reusable RBAC snapshot with read-only kubectl calls
    Snapshot(SnapshotArgs),
    /// Evaluate a subject and matrix, then write Markdown and JSON evidence
    Report(ReportArgs),
    /// Generate a new local Ed25519 signing key
    Keygen(KeygenArgs),
    /// Verify an evidence JSON packet's embedded signature
    Verify(VerifyArgs),
}

#[derive(Args)]
struct ClusterArgs {
    /// kubectl context to use (the current context when omitted)
    #[arg(long)]
    context: Option<String>,
    /// Explicit kubeconfig path passed through to kubectl; its contents are never read by kpe
    #[arg(long)]
    kubeconfig: Option<PathBuf>,
}

#[derive(Args)]
struct SnapshotArgs {
    #[command(flatten)]
    cluster: ClusterArgs,
    /// Snapshot path; required unless --json is used
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Print JSON to stdout instead of writing a file
    #[arg(long, conflicts_with = "output")]
    json: bool,
}

#[derive(Args)]
struct ReportArgs {
    /// Subject: user:<name>, group:<name>, or serviceaccount:<namespace>:<name>
    #[arg(short, long)]
    subject: String,
    /// Assert an identity-provider group; repeat for multiple groups
    #[arg(long = "as-group")]
    groups: Vec<String>,
    /// JSON access matrix (see README)
    #[arg(short, long)]
    matrix: PathBuf,
    /// Evaluate an existing snapshot instead of contacting a cluster
    #[arg(long)]
    snapshot: Option<PathBuf>,
    #[command(flatten)]
    cluster: ClusterArgs,
    /// Output basename; writes <name>.md and <name>.json
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Print JSON to stdout instead of writing files
    #[arg(long, conflicts_with = "output")]
    json: bool,
    /// Sign the JSON packet with a 32-byte hex Ed25519 seed
    #[arg(long)]
    signing_key: Option<PathBuf>,
    /// Exit 3 if a result of this kind occurs
    #[arg(long, value_enum)]
    fail_on: Option<FailOn>,
}

#[derive(Clone, Copy, ValueEnum)]
enum FailOn {
    Allowed,
    Denied,
}

#[derive(Args)]
struct KeygenArgs {
    /// New key path (existing files are never overwritten)
    #[arg(short, long)]
    output: PathBuf,
}

#[derive(Args)]
struct VerifyArgs {
    /// Signed evidence JSON file
    packet: PathBuf,
    /// Emit a small machine-readable verification result
    #[arg(long)]
    json: bool,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<u8> {
    match cli.command {
        Command::Snapshot(args) => snapshot_command(args),
        Command::Report(args) => report_command(args),
        Command::Keygen(args) => {
            packet::generate_key(&args.output)?;
            eprintln!(
                "Created {}. Keep this signing key private.",
                args.output.display()
            );
            Ok(0)
        }
        Command::Verify(args) => verify_command(args),
    }
}

fn snapshot_command(args: SnapshotArgs) -> Result<u8> {
    if !args.json && args.output.is_none() {
        bail!("pass --output <file> or --json");
    }
    let snapshot = collector::collect(
        args.cluster.context.as_deref(),
        args.cluster.kubeconfig.as_deref(),
    )?;
    let json = serde_json::to_string_pretty(&snapshot)?;
    if args.json {
        println!("{json}");
    } else if let Some(path) = args.output {
        write_parented(&path, json.as_bytes())?;
        eprintln!(
            "Collected {} RBAC objects into {}.",
            snapshot.roles.len()
                + snapshot.cluster_roles.len()
                + snapshot.role_bindings.len()
                + snapshot.cluster_role_bindings.len(),
            path.display()
        );
    }
    Ok(0)
}

fn report_command(args: ReportArgs) -> Result<u8> {
    if !args.json && args.output.is_none() {
        bail!("pass --output <basename> or --json");
    }
    let subject = parse_subject(&args.subject).map_err(anyhow::Error::msg)?;
    let matrix: AccessMatrix = read_json(&args.matrix, "access matrix")?;
    for (index, check) in matrix.checks.iter().enumerate() {
        check
            .validate()
            .map_err(|message| anyhow::anyhow!("matrix check {}: {message}", index + 1))?;
    }
    let snapshot = match args.snapshot {
        Some(path) => read_json(&path, "RBAC snapshot")?,
        None => collector::collect(
            args.cluster.context.as_deref(),
            args.cluster.kubeconfig.as_deref(),
        )?,
    };
    validate_snapshot(&snapshot)?;
    let mut report = evaluate(&snapshot, &subject, &args.groups, &matrix);
    if let Some(key) = args.signing_key {
        packet::sign(&mut report, &key)?;
    }
    let json = serde_json::to_string_pretty(&report)?;
    if args.json {
        println!("{json}");
    } else if let Some(base) = args.output {
        let json_path = with_extension(&base, "json");
        let markdown_path = with_extension(&base, "md");
        write_parented(&json_path, json.as_bytes())?;
        write_parented(&markdown_path, render::markdown(&report).as_bytes())?;
        eprintln!(
            "Wrote {} and {} ({} allowed, {} denied, {} uncertain).",
            markdown_path.display(),
            json_path.display(),
            report.summary.allowed,
            report.summary.denied,
            report.summary.uncertain
        );
    }
    let triggered = match args.fail_on {
        Some(FailOn::Allowed) => report.summary.allowed > 0,
        Some(FailOn::Denied) => report.summary.denied > 0,
        None => false,
    };
    Ok(if triggered { 3 } else { 0 })
}

fn verify_command(args: VerifyArgs) -> Result<u8> {
    let report: EvidenceReport = read_json(&args.packet, "evidence packet")?;
    packet::verify(&report)?;
    if args.json {
        println!(
            "{{\"valid\":true,\"subject\":{}}}",
            serde_json::to_string(&report.subject.label())?
        );
    } else {
        println!(
            "VALID — Ed25519 signature and SHA-256 digest match for {}.",
            report.subject.label()
        );
    }
    Ok(0)
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, description: &str) -> Result<T> {
    let bytes = fs::read(path)
        .with_context(|| format!("could not read {description} {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("{description} {} is not valid JSON", path.display()))
}

fn validate_snapshot(snapshot: &Snapshot) -> Result<()> {
    if snapshot.schema_version != "kpe.snapshot/v1" {
        bail!(
            "unsupported snapshot schema {:?}; expected kpe.snapshot/v1",
            snapshot.schema_version
        );
    }
    Ok(())
}

fn with_extension(base: &Path, extension: &str) -> PathBuf {
    let mut result = base.to_path_buf();
    result.set_extension(extension);
    result
}

fn write_parented(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("could not create {}", parent.display()))?;
    }
    fs::write(path, bytes).with_context(|| format!("could not write {}", path.display()))
}
