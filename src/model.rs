use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ObjectMeta {
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyRule {
    #[serde(default)]
    pub api_groups: Vec<String>,
    #[serde(default)]
    pub resources: Vec<String>,
    #[serde(default)]
    pub verbs: Vec<String>,
    #[serde(default)]
    pub resource_names: Vec<String>,
    #[serde(default)]
    #[serde(rename = "nonResourceURLs", alias = "nonResourceUrls")]
    pub non_resource_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    #[serde(default)]
    pub kind: String,
    pub metadata: ObjectMeta,
    #[serde(default)]
    pub rules: Vec<PolicyRule>,
    #[serde(default)]
    pub aggregation_rule: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RbacSubject {
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub api_group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleRef {
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub api_group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    #[serde(default)]
    pub kind: String,
    pub metadata: ObjectMeta,
    #[serde(default)]
    pub subjects: Vec<RbacSubject>,
    pub role_ref: RoleRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub schema_version: String,
    pub collected_at: String,
    pub collector_version: String,
    pub context: String,
    pub server_version: String,
    #[serde(default)]
    pub roles: Vec<Role>,
    #[serde(default)]
    pub cluster_roles: Vec<Role>,
    #[serde(default)]
    pub role_bindings: Vec<Binding>,
    #[serde(default)]
    pub cluster_role_bindings: Vec<Binding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubeList<T> {
    pub items: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessMatrix {
    pub checks: Vec<AccessCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AccessCheck {
    pub verb: String,
    #[serde(default)]
    pub api_group: String,
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub subresource: Option<String>,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub resource_name: Option<String>,
    #[serde(default)]
    #[serde(rename = "nonResourceURL", alias = "nonResourceUrl")]
    pub non_resource_url: Option<String>,
}

impl AccessCheck {
    pub fn validate(&self) -> Result<(), String> {
        if self.verb.trim().is_empty() {
            return Err("verb must not be empty".into());
        }
        match (&self.resource, &self.non_resource_url) {
            (Some(_), Some(_)) => Err("set resource or nonResourceURL, not both".into()),
            (None, None) => Err("set resource or nonResourceURL".into()),
            _ if self.subresource.is_some() && self.resource.is_none() => {
                Err("subresource requires resource".into())
            }
            _ => Ok(()),
        }
    }

    pub fn target(&self) -> String {
        if let Some(url) = &self.non_resource_url {
            return url.clone();
        }
        let mut target = self.resource.clone().unwrap_or_default();
        if let Some(subresource) = &self.subresource {
            target.push('/');
            target.push_str(subresource);
        }
        if !self.api_group.is_empty() {
            target.push('.');
            target.push_str(&self.api_group);
        }
        if let Some(name) = &self.resource_name {
            target.push('/');
            target.push_str(name);
        }
        target
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SubjectIdentity {
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
}

impl SubjectIdentity {
    pub fn label(&self) -> String {
        match &self.namespace {
            Some(namespace) => format!("{}:{namespace}:{}", self.kind.to_lowercase(), self.name),
            None => format!("{}:{}", self.kind.to_lowercase(), self.name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceReport {
    pub schema_version: String,
    pub generated_at: String,
    pub tool_version: String,
    pub subject: SubjectIdentity,
    pub evaluated_groups: Vec<String>,
    pub source: EvidenceSource,
    pub summary: EvidenceSummary,
    pub checks: Vec<CheckResult>,
    pub limitations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<PacketSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceSource {
    pub collected_at: String,
    pub context: String,
    pub server_version: String,
    pub snapshot_sha256: String,
    pub role_count: usize,
    pub cluster_role_count: usize,
    pub role_binding_count: usize,
    pub cluster_role_binding_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSummary {
    pub total: usize,
    pub allowed: usize,
    pub denied: usize,
    pub uncertain: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub request: AccessCheck,
    pub allowed: bool,
    pub uncertain: bool,
    pub grants: Vec<GrantProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantProof {
    pub binding_kind: String,
    pub binding_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_namespace: Option<String>,
    pub matched_subject: RbacSubject,
    pub role_kind: String,
    pub role_name: String,
    pub rule_index: usize,
    pub rule: PolicyRule,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PacketSignature {
    pub algorithm: String,
    pub public_key: String,
    pub signature: String,
    pub content_sha256: String,
}
