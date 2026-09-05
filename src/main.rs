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
    /// Run the bundled sample without kubectl or a cluster
    Demo(DemoArgs),
    /// Verify a packet signature and, optionally, an approved signer
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
    /// Write the matching Base64 public key for separate delivery to an auditor
    #[arg(long)]
    public_key_output: Option<PathBuf>,
}

#[derive(Args)]
struct DemoArgs {
    /// New directory for sample inputs and evidence (a temporary directory when omitted)
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Args)]
struct VerifyArgs {
    /// Signed evidence JSON file
    packet: PathBuf,
    /// Emit a small machine-readable verification result
    #[arg(long)]
    json: bool,
    /// Require the packet signer to match this Base64 public-key file
    #[arg(long, conflicts_with = "trusted_fingerprint")]
    trusted_public_key: Option<PathBuf>,
    /// Require this SHA-256 public-key fingerprint
    #[arg(long, conflicts_with = "trusted_public_key")]
    trusted_fingerprint: Option<String>,
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
            if args
                .public_key_output
                .as_ref()
                .is_some_and(|path| path.exists())
            {
                bail!("refusing to overwrite public key output");
            }
            let identity = packet::generate_key(&args.output)?;
            if let Some(path) = args.public_key_output {
                packet::write_public_key(&path, &identity.public_key)?;
                eprintln!("Created public key {}.", path.display());
            }
            eprintln!(
                "Created {}. Keep this signing key private.\nSigner fingerprint: {}",
                args.output.display(),
                identity.fingerprint
            );
            Ok(0)
        }
        Command::Demo(args) => demo_command(args),
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
    let signer = packet::verify(&report)?;
    let trusted_signer = args.trusted_public_key.is_some() || args.trusted_fingerprint.is_some();
    if let Some(path) = args.trusted_public_key {
        packet::require_public_key(&signer, &path)?;
    }
    if let Some(fingerprint) = args.trusted_fingerprint {
        packet::require_fingerprint(&signer, &fingerprint)?;
    }
    if args.json {
        println!(
            "{{\"valid\":true,\"trustedSigner\":{},\"subject\":{},\"signerFingerprint\":{}}}",
            trusted_signer,
            serde_json::to_string(&report.subject.label())?,
            serde_json::to_string(&signer.fingerprint)?
        );
    } else if trusted_signer {
        println!(
            "VALID — signature, digest, and trusted signer match for {}. Signer: {}.",
            report.subject.label(),
            signer.fingerprint
        );
    } else {
        println!(
            "VALID CONTENT — signature and digest match for {}. Signer: {}. Trust was not checked; pass --trusted-public-key or --trusted-fingerprint.",
            report.subject.label(),
            signer.fingerprint
        );
    }
    Ok(0)
}

fn demo_command(args: DemoArgs) -> Result<u8> {
    let snapshot: Snapshot = serde_json::from_str(include_str!("../examples/rbac-snapshot.json"))
        .context("bundled demo snapshot is invalid")?;
    let matrix: AccessMatrix = serde_json::from_str(include_str!("../examples/matrix.json"))
        .context("bundled demo matrix is invalid")?;
    let subject = parse_subject("user:alice@example.com").map_err(anyhow::Error::msg)?;
    let report = evaluate(&snapshot, &subject, &["platform-engineers".into()], &matrix);
    let directory = args.output.unwrap_or_else(|| {
        std::env::temp_dir().join(format!(
            "kpe-demo-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_millis()
        ))
    });
    fs::create_dir(&directory).with_context(|| {
        format!(
            "could not create demo directory {} (choose a path that does not exist)",
            directory.display()
        )
    })?;
    write_parented(
        &directory.join("rbac-snapshot.json"),
        include_bytes!("../examples/rbac-snapshot.json"),
    )?;
    write_parented(
        &directory.join("matrix.json"),
        include_bytes!("../examples/matrix.json"),
    )?;
    write_parented(
        &directory.join("evidence.json"),
        serde_json::to_string_pretty(&report)?.as_bytes(),
    )?;
    write_parented(
        &directory.join("evidence.md"),
        render::markdown(&report).as_bytes(),
    )?;
    println!("Demo — bundled sample data; no cluster was contacted.");
    println!("Subject: {}", report.subject.label());
    println!(
        "Result: {} checks — {} allowed, {} denied, {} uncertain.",
        report.summary.total,
        report.summary.allowed,
        report.summary.denied,
        report.summary.uncertain
    );
    println!("Sample files: {}", directory.display());
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
