use crate::model::{Binding, KubeList, Role, Snapshot};
use anyhow::{Context, Result, bail};
use chrono::Utc;
use serde::de::DeserializeOwned;
use std::path::Path;
use std::process::Command;

pub fn collect(context: Option<&str>, kubeconfig: Option<&Path>) -> Result<Snapshot> {
    ensure_kubectl()?;
    let roles = get_list::<Role>("roles", true, context, kubeconfig)?;
    let role_bindings = get_list::<Binding>("rolebindings", true, context, kubeconfig)?;
    let cluster_roles = get_list::<Role>("clusterroles", false, context, kubeconfig)?;
    let cluster_role_bindings =
        get_list::<Binding>("clusterrolebindings", false, context, kubeconfig)?;
    let selected_context = match context {
        Some(value) => value.to_owned(),
        None => kubectl_text(&["config", "current-context"], None, kubeconfig)
            .unwrap_or_else(|_| "unknown".into()),
    };
    let version: serde_json::Value = serde_json::from_str(
        &kubectl_text(&["version", "-o", "json"], context, kubeconfig)
            .context("could not query Kubernetes version")?,
    )
    .context("kubectl version returned invalid JSON")?;
    let server_version = version
        .pointer("/serverVersion/gitVersion")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown")
        .to_owned();

    Ok(Snapshot {
        schema_version: "kpe.snapshot/v1".into(),
        collected_at: Utc::now().to_rfc3339(),
        collector_version: env!("CARGO_PKG_VERSION").into(),
        context: selected_context.trim().into(),
        server_version,
        roles,
        cluster_roles,
        role_bindings,
        cluster_role_bindings,
    })
}

fn ensure_kubectl() -> Result<()> {
    match Command::new("kubectl")
        .arg("version")
        .arg("--client")
        .arg("-o")
        .arg("json")
        .output()
    {
        Ok(output) if output.status.success() => Ok(()),
        Ok(_) => bail!("kubectl is installed but not usable; run `kubectl version --client`"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bail!(
                "kubectl was not found; install kubectl or pass --snapshot for offline evaluation"
            )
        }
        Err(error) => Err(error).context("could not start kubectl"),
    }
}

fn get_list<T: DeserializeOwned>(
    resource: &str,
    all_namespaces: bool,
    context: Option<&str>,
    kubeconfig: Option<&Path>,
) -> Result<Vec<T>> {
    let mut args = vec!["get", resource];
    if all_namespaces {
        args.push("--all-namespaces");
    }
    args.extend(["-o", "json"]);
    let raw = kubectl_text(&args, context, kubeconfig)
        .with_context(|| format!("read-only collection failed for {resource}"))?;
    let list: KubeList<T> = serde_json::from_str(&raw)
        .with_context(|| format!("kubectl returned invalid JSON for {resource}"))?;
    Ok(list.items)
}

fn kubectl_text(args: &[&str], context: Option<&str>, kubeconfig: Option<&Path>) -> Result<String> {
    let mut command = Command::new("kubectl");
    command.args(args);
    if let Some(context) = context {
        command.args(["--context", context]);
    }
    if let Some(path) = kubeconfig {
        command.arg("--kubeconfig").arg(path);
    }
    let output = command.output().context("could not run kubectl")?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr);
        bail!("kubectl exited with {}: {}", output.status, message.trim());
    }
    String::from_utf8(output.stdout).context("kubectl output was not UTF-8")
}
