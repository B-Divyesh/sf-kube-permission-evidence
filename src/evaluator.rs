use crate::model::*;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub fn parse_subject(value: &str) -> Result<SubjectIdentity, String> {
    if let Some(name) = value.strip_prefix("user:").filter(|name| !name.is_empty()) {
        return Ok(SubjectIdentity {
            kind: "User".into(),
            name: name.into(),
            namespace: None,
        });
    }
    if let Some(name) = value.strip_prefix("group:").filter(|name| !name.is_empty()) {
        return Ok(SubjectIdentity {
            kind: "Group".into(),
            name: name.into(),
            namespace: None,
        });
    }
    let value = value
        .strip_prefix("serviceaccount:")
        .or_else(|| value.strip_prefix("sa:"));
    if let Some(value) = value {
        if let Some((namespace, name)) = value.split_once(':')
            && !namespace.is_empty()
            && !name.is_empty()
            && !name.contains(':')
        {
            Ok(SubjectIdentity {
                kind: "ServiceAccount".into(),
                name: name.into(),
                namespace: Some(namespace.into()),
            })
        } else {
            Err("expected serviceaccount:<namespace>:<name>".into())
        }
    } else {
        Err("expected user:<name>, group:<name>, or serviceaccount:<namespace>:<name>".into())
    }
}

pub fn evaluated_groups(subject: &SubjectIdentity, asserted: &[String]) -> Vec<String> {
    let mut groups: BTreeSet<String> = asserted
        .iter()
        .filter(|v| !v.trim().is_empty())
        .cloned()
        .collect();
    match subject.kind.as_str() {
        "ServiceAccount" => {
            groups.insert("system:authenticated".into());
            groups.insert("system:serviceaccounts".into());
            if let Some(namespace) = &subject.namespace {
                groups.insert(format!("system:serviceaccounts:{namespace}"));
            }
        }
        "User" if subject.name != "system:anonymous" => {
            groups.insert("system:authenticated".into());
        }
        _ => {}
    }
    groups.into_iter().collect()
}

pub fn evaluate(
    snapshot: &Snapshot,
    subject: &SubjectIdentity,
    asserted_groups: &[String],
    matrix: &AccessMatrix,
) -> EvidenceReport {
    let groups = evaluated_groups(subject, asserted_groups);
    let mut checks = Vec::with_capacity(matrix.checks.len());
    for request in &matrix.checks {
        let mut grants = Vec::new();
        evaluate_bindings(
            snapshot,
            subject,
            &groups,
            request,
            &snapshot.role_bindings,
            false,
            &mut grants,
        );
        evaluate_bindings(
            snapshot,
            subject,
            &groups,
            request,
            &snapshot.cluster_role_bindings,
            true,
            &mut grants,
        );
        grants.sort_by(|a, b| {
            (
                &a.binding_kind,
                &a.binding_namespace,
                &a.binding_name,
                &a.role_name,
                a.rule_index,
            )
                .cmp(&(
                    &b.binding_kind,
                    &b.binding_namespace,
                    &b.binding_name,
                    &b.role_name,
                    b.rule_index,
                ))
        });
        checks.push(CheckResult {
            allowed: !grants.is_empty(),
            uncertain: grants.iter().any(|grant| grant.uncertainty.is_some()),
            request: request.clone(),
            grants,
        });
    }
    let allowed = checks.iter().filter(|result| result.allowed).count();
    let uncertain = checks.iter().filter(|result| result.uncertain).count();
    let snapshot_bytes = serde_json::to_vec(snapshot).expect("snapshot serialization cannot fail");
    EvidenceReport {
        schema_version: "kpe.evidence/v1".into(),
        generated_at: Utc::now().to_rfc3339(),
        tool_version: env!("CARGO_PKG_VERSION").into(),
        subject: subject.clone(),
        evaluated_groups: groups,
        source: EvidenceSource {
            collected_at: snapshot.collected_at.clone(),
            context: snapshot.context.clone(),
            server_version: snapshot.server_version.clone(),
            snapshot_sha256: hex::encode(Sha256::digest(snapshot_bytes)),
            role_count: snapshot.roles.len(),
            cluster_role_count: snapshot.cluster_roles.len(),
            role_binding_count: snapshot.role_bindings.len(),
            cluster_role_binding_count: snapshot.cluster_role_bindings.len(),
        },
        summary: EvidenceSummary { total: checks.len(), allowed, denied: checks.len() - allowed, uncertain },
        checks,
        limitations: vec![
            "This packet evaluates Kubernetes RBAC objects only; webhook, Node, and other authorizers are not represented.".into(),
            "External identity-provider group membership is included only when supplied with --as-group.".into(),
            "Aggregated ClusterRole rules are controller-resolved and marked uncertain because results can vary with installed APIs and Kubernetes version.".into(),
            "Rules constrained by resourceNames never grant top-level create or deletecollection checks; named list/watch checks require a matching metadata.name fieldSelector.".into(),
        ],
        signature: None,
    }
}

fn evaluate_bindings(
    snapshot: &Snapshot,
    subject: &SubjectIdentity,
    groups: &[String],
    request: &AccessCheck,
    bindings: &[Binding],
    cluster_binding: bool,
    grants: &mut Vec<GrantProof>,
) {
    for binding in bindings {
        // Non-resource URLs have no namespace. Kubernetes only applies those
        // rules through a ClusterRoleBinding, never through a RoleBinding.
        // Keep this guard here as well as matrix validation because callers of
        // the public evaluator can construct AccessCheck values directly.
        if !cluster_binding && request.non_resource_url.is_some() {
            continue;
        }
        if !cluster_binding && request.namespace.as_deref() != binding.metadata.namespace.as_deref()
        {
            continue;
        }
        let matched_subjects: Vec<&RbacSubject> = binding
            .subjects
            .iter()
            .filter(|candidate| {
                subject_matches(
                    candidate,
                    binding.metadata.namespace.as_deref(),
                    subject,
                    groups,
                )
            })
            .collect();
        if matched_subjects.is_empty() {
            continue;
        }

        let role = if binding.role_ref.kind == "Role" && !cluster_binding {
            snapshot.roles.iter().find(|role| {
                role.metadata.name == binding.role_ref.name
                    && role.metadata.namespace == binding.metadata.namespace
            })
        } else if binding.role_ref.kind == "ClusterRole" {
            snapshot
                .cluster_roles
                .iter()
                .find(|role| role.metadata.name == binding.role_ref.name)
        } else {
            None
        };
        let Some(role) = role else {
            continue;
        };
        for (rule_index, rule) in role.rules.iter().enumerate() {
            if !rule_matches(rule, request) {
                continue;
            }
            for matched_subject in &matched_subjects {
                grants.push(GrantProof {
                    binding_kind: if cluster_binding { "ClusterRoleBinding".into() } else { "RoleBinding".into() },
                    binding_name: binding.metadata.name.clone(),
                    binding_namespace: binding.metadata.namespace.clone(),
                    matched_subject: (*matched_subject).clone(),
                    role_kind: binding.role_ref.kind.clone(),
                    role_name: binding.role_ref.name.clone(),
                    rule_index,
                    rule: rule.clone(),
                    uncertainty: role.aggregation_rule.as_ref().map(|_| {
                        format!("ClusterRole {} uses aggregationRule; evaluated rules are the API server's collected controller result", role.metadata.name)
                    }),
                });
            }
        }
    }
}

fn subject_matches(
    candidate: &RbacSubject,
    binding_namespace: Option<&str>,
    subject: &SubjectIdentity,
    groups: &[String],
) -> bool {
    match candidate.kind.as_str() {
        "User" => subject.kind == "User" && candidate.name == subject.name,
        "Group" => {
            (subject.kind == "Group" && candidate.name == subject.name)
                || groups.iter().any(|group| group == &candidate.name)
        }
        "ServiceAccount" => {
            subject.kind == "ServiceAccount"
                && candidate.name == subject.name
                && candidate.namespace.as_deref().or(binding_namespace)
                    == subject.namespace.as_deref()
        }
        _ => false,
    }
}

fn rule_matches(rule: &PolicyRule, request: &AccessCheck) -> bool {
    if !contains_or_star(&rule.verbs, &request.verb) {
        return false;
    }
    if let Some(url) = &request.non_resource_url {
        return rule
            .non_resource_urls
            .iter()
            .any(|pattern| url_matches(pattern, url));
    }
    if !rule.non_resource_urls.is_empty() && rule.resources.is_empty() {
        return false;
    }
    if !contains_or_star(&rule.api_groups, &request.api_group) {
        return false;
    }
    let Some(resource) = request.resource.as_deref() else {
        return false;
    };
    if !resource_matches(&rule.resources, resource, request.subresource.as_deref()) {
        return false;
    }
    if rule.resource_names.is_empty() {
        return true;
    }
    let Some(name) = request
        .resource_name
        .as_ref()
        .filter(|name| rule.resource_names.iter().any(|allowed| allowed == *name))
    else {
        return false;
    };

    match request.verb.as_str() {
        "create" if request.subresource.is_none() => false,
        "deletecollection" => false,
        "list" | "watch" => request
            .field_selector
            .as_deref()
            .is_some_and(|selector| field_selector_matches_name(selector, name)),
        _ => true,
    }
}

fn field_selector_matches_name(selector: &str, resource_name: &str) -> bool {
    selector.split(',').map(str::trim).any(|requirement| {
        requirement
            .strip_prefix("metadata.name==")
            .or_else(|| requirement.strip_prefix("metadata.name="))
            .is_some_and(|selected_name| selected_name == resource_name)
    })
}

fn contains_or_star(values: &[String], expected: &str) -> bool {
    values.iter().any(|value| value == "*" || value == expected)
}

/// Mirrors Kubernetes RBAC's ResourceMatches behavior, including the special
/// `*/subresource` form (for example, `*/scale`). A wildcard subresource must
/// not match the parent resource or a different subresource.
fn resource_matches(values: &[String], resource: &str, subresource: Option<&str>) -> bool {
    let requested = subresource
        .map(|subresource| format!("{resource}/{subresource}"))
        .unwrap_or_else(|| resource.to_owned());

    values.iter().any(|value| {
        value == "*"
            || value == &requested
            || subresource.is_some_and(|subresource| value == &format!("*/{subresource}"))
    })
}

fn url_matches(pattern: &str, url: &str) -> bool {
    pattern == "*"
        || pattern == url
        || pattern
            .strip_suffix('*')
            .is_some_and(|prefix| url.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_documented_subjects() {
        assert_eq!(
            parse_subject("user:alice@example.com").unwrap().name,
            "alice@example.com"
        );
        assert_eq!(parse_subject("user:oidc:alice").unwrap().name, "oidc:alice");
        assert_eq!(
            parse_subject("serviceaccount:payments:worker")
                .unwrap()
                .namespace
                .as_deref(),
            Some("payments")
        );
        assert!(parse_subject("serviceaccount:broken").is_err());
    }

    #[test]
    fn subresources_and_named_resources_are_exact() {
        let rule = PolicyRule {
            api_groups: vec!["".into()],
            resources: vec!["pods/exec".into()],
            verbs: vec!["create".into()],
            resource_names: vec!["shell".into()],
            ..Default::default()
        };
        let mut check = AccessCheck {
            verb: "create".into(),
            api_group: "".into(),
            resource: Some("pods".into()),
            subresource: Some("exec".into()),
            namespace: Some("dev".into()),
            resource_name: Some("shell".into()),
            field_selector: None,
            non_resource_url: None,
        };
        assert!(rule_matches(&rule, &check));
        check.resource_name = None;
        assert!(!rule_matches(&rule, &check));
    }

    #[test]
    fn wildcard_subresource_matches_only_that_subresource() {
        let rule = PolicyRule {
            api_groups: vec!["apps".into()],
            resources: vec!["*/scale".into()],
            verbs: vec!["update".into()],
            ..Default::default()
        };
        let mut check = AccessCheck {
            verb: "update".into(),
            api_group: "apps".into(),
            resource: Some("deployments".into()),
            subresource: Some("scale".into()),
            namespace: Some("payments".into()),
            resource_name: None,
            field_selector: None,
            non_resource_url: None,
        };

        assert!(rule_matches(&rule, &check));
        check.resource = Some("statefulsets".into());
        assert!(rule_matches(&rule, &check));
        check.subresource = Some("status".into());
        assert!(!rule_matches(&rule, &check));
        check.subresource = None;
        assert!(!rule_matches(&rule, &check));
    }

    #[test]
    fn non_resource_url_supports_trailing_wildcard_only() {
        let rule = PolicyRule {
            verbs: vec!["get".into()],
            non_resource_urls: vec!["/healthz/*".into()],
            ..Default::default()
        };
        let check = AccessCheck {
            verb: "get".into(),
            api_group: "".into(),
            resource: None,
            subresource: None,
            namespace: None,
            resource_name: None,
            field_selector: None,
            non_resource_url: Some("/healthz/ready".into()),
        };
        assert!(rule_matches(&rule, &check));
    }
}
