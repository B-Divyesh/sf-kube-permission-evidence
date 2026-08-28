use crate::model::*;

pub fn markdown(report: &EvidenceReport) -> String {
    let mut out = String::new();
    out.push_str("# Kubernetes permission evidence\n\n");
    out.push_str(&format!("**Subject:** `{}`  \n", report.subject.label()));
    out.push_str(&format!(
        "**Collected:** {}  \n",
        report.source.collected_at
    ));
    out.push_str(&format!(
        "**Context:** `{}`  \n",
        escape_inline(&report.source.context)
    ));
    out.push_str(&format!(
        "**Kubernetes:** `{}`  \n",
        escape_inline(&report.source.server_version)
    ));
    out.push_str(&format!(
        "**Snapshot SHA-256:** `{}`  \n",
        report.source.snapshot_sha256
    ));
    if let Some(signature) = &report.signature {
        out.push_str(&format!(
            "**Packet signature:** Ed25519 `{}`…  \n",
            &signature.signature[..16.min(signature.signature.len())]
        ));
        out.push_str(&format!(
            "**Signed content SHA-256:** `{}`  \n",
            signature.content_sha256
        ));
    } else {
        out.push_str("**Packet signature:** Not signed  \n");
    }
    out.push_str("\n## Summary\n\n");
    out.push_str(&format!(
        "{} checks: **{} allowed**, **{} denied**, **{} uncertain**.\n\n",
        report.summary.total,
        report.summary.allowed,
        report.summary.denied,
        report.summary.uncertain
    ));
    out.push_str("| # | Decision | Verb | Resource or URL | Namespace | Grant paths |\n");
    out.push_str("|---:|---|---|---|---|---:|\n");
    for (index, result) in report.checks.iter().enumerate() {
        let decision = if result.allowed && result.uncertain {
            "ALLOWED ⚠"
        } else if result.allowed {
            "ALLOWED"
        } else {
            "DENIED"
        };
        out.push_str(&format!(
            "| {} | **{}** | `{}` | `{}` | {} | {} |\n",
            index + 1,
            decision,
            escape_table(&result.request.verb),
            escape_table(&result.request.target()),
            result
                .request
                .namespace
                .as_deref()
                .map(|v| format!("`{}`", escape_table(v)))
                .unwrap_or_else(|| "cluster / non-resource".into()),
            result.grants.len()
        ));
    }

    out.push_str("\n## Causal proof\n\n");
    if report.checks.is_empty() {
        out.push_str("The supplied matrix contained no checks.\n\n");
    }
    for (index, result) in report.checks.iter().enumerate() {
        let namespace = result
            .request
            .namespace
            .as_deref()
            .unwrap_or("cluster / non-resource");
        out.push_str(&format!(
            "### {}. {} `{}` in `{}`\n\n",
            index + 1,
            result.request.verb,
            result.request.target(),
            namespace
        ));
        if result.grants.is_empty() {
            out.push_str(
                "**DENIED** — no collected RBAC binding and rule matched this request.\n\n",
            );
            continue;
        }
        out.push_str(if result.uncertain {
            "**ALLOWED, WITH UNCERTAINTY**\n\n"
        } else {
            "**ALLOWED**\n\n"
        });
        for (grant_index, grant) in result.grants.iter().enumerate() {
            let binding = match &grant.binding_namespace {
                Some(namespace) => format!(
                    "{}/{} in {}",
                    grant.binding_kind, grant.binding_name, namespace
                ),
                None => format!("{}/{}", grant.binding_kind, grant.binding_name),
            };
            out.push_str(&format!(
                "{}. `{}` matches `{}` → `{}/{}` rule {}\n\n",
                grant_index + 1,
                grant.matched_subject.kind,
                binding,
                grant.role_kind,
                grant.role_name,
                grant.rule_index
            ));
            out.push_str("   ```json\n");
            let rule =
                serde_json::to_string_pretty(&grant.rule).expect("rule serialization cannot fail");
            for line in rule.lines() {
                out.push_str("   ");
                out.push_str(line);
                out.push('\n');
            }
            out.push_str("   ```\n\n");
            if let Some(note) = &grant.uncertainty {
                out.push_str(&format!("   ⚠ **Uncertain:** {note}.\n\n"));
            }
        }
    }
    out.push_str("## Scope and limitations\n\n");
    out.push_str(&format!(
        "Inventory: {} Roles, {} ClusterRoles, {} RoleBindings, {} ClusterRoleBindings.\n\n",
        report.source.role_count,
        report.source.cluster_role_count,
        report.source.role_binding_count,
        report.source.cluster_role_binding_count
    ));
    if !report.evaluated_groups.is_empty() {
        out.push_str("Evaluated group memberships: ");
        out.push_str(
            &report
                .evaluated_groups
                .iter()
                .map(|group| format!("`{}`", escape_inline(group)))
                .collect::<Vec<_>>()
                .join(", "),
        );
        out.push_str(".\n\n");
    }
    for limitation in &report.limitations {
        out.push_str("- ");
        out.push_str(limitation);
        out.push('\n');
    }
    out.push_str("\n---\nGenerated by Kube Permission Evidence. Verify the JSON companion with `kpe verify`.\n");
    out
}

fn escape_table(value: &str) -> String {
    value.replace('|', "\\|").replace('`', "\\`")
}
fn escape_inline(value: &str) -> String {
    value.replace('`', "\\`")
}
