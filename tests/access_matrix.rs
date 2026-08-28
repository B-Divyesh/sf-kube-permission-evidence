use kube_permission_evidence::*;

fn metadata(name: &str, namespace: Option<&str>) -> ObjectMeta {
    ObjectMeta {
        name: name.into(),
        namespace: namespace.map(str::to_owned),
        labels: Default::default(),
    }
}

fn rule(groups: &[&str], resources: &[&str], verbs: &[&str]) -> PolicyRule {
    PolicyRule {
        api_groups: groups.iter().map(|v| (*v).into()).collect(),
        resources: resources.iter().map(|v| (*v).into()).collect(),
        verbs: verbs.iter().map(|v| (*v).into()).collect(),
        ..Default::default()
    }
}

fn subject(kind: &str, name: &str, namespace: Option<&str>) -> RbacSubject {
    RbacSubject {
        kind: kind.into(),
        name: name.into(),
        namespace: namespace.map(str::to_owned),
        api_group: None,
    }
}

fn binding(
    kind: &str,
    name: &str,
    namespace: Option<&str>,
    subjects: Vec<RbacSubject>,
    role_kind: &str,
    role_name: &str,
) -> Binding {
    Binding {
        kind: kind.into(),
        metadata: metadata(name, namespace),
        subjects,
        role_ref: RoleRef {
            kind: role_kind.into(),
            name: role_name.into(),
            api_group: Some("rbac.authorization.k8s.io".into()),
        },
    }
}

fn check(verb: &str, group: &str, resource: &str, namespace: Option<&str>) -> AccessCheck {
    AccessCheck {
        verb: verb.into(),
        api_group: group.into(),
        resource: Some(resource.into()),
        subresource: None,
        namespace: namespace.map(str::to_owned),
        resource_name: None,
        field_selector: None,
        non_resource_url: None,
    }
}

#[test]
fn seeded_twenty_case_matrix_is_explained_exactly() {
    let mut exec_rule = rule(&[""], &["pods/exec"], &["create"]);
    exec_rule.resource_names = vec!["toolbox".into()];
    let snapshot = Snapshot {
        schema_version: "kpe.snapshot/v1".into(),
        collected_at: "2026-08-28T00:00:00Z".into(),
        collector_version: "0.1.0".into(),
        context: "seeded".into(),
        server_version: "v1.33.4".into(),
        roles: vec![
            Role {
                kind: "Role".into(),
                metadata: metadata("dev-pods", Some("dev")),
                rules: vec![rule(&[""], &["pods"], &["get", "list"]), exec_rule],
                aggregation_rule: None,
            },
            Role {
                kind: "Role".into(),
                metadata: metadata("prod-config", Some("prod")),
                rules: vec![rule(&[""], &["configmaps"], &["get"])],
                aggregation_rule: None,
            },
        ],
        cluster_roles: vec![
            Role {
                kind: "ClusterRole".into(),
                metadata: metadata("secret-reader", None),
                rules: vec![rule(&[""], &["secrets"], &["get", "list"])],
                aggregation_rule: None,
            },
            Role {
                kind: "ClusterRole".into(),
                metadata: metadata("global-observer", None),
                rules: vec![
                    rule(&["apps"], &["deployments"], &["get", "list", "watch"]),
                    rule(&[""], &["namespaces"], &["get"]),
                    PolicyRule {
                        verbs: vec!["get".into()],
                        non_resource_urls: vec!["/healthz/*".into()],
                        ..Default::default()
                    },
                ],
                aggregation_rule: None,
            },
            Role {
                kind: "ClusterRole".into(),
                metadata: metadata("aggregate-workloads", None),
                rules: vec![rule(&["batch"], &["jobs"], &["get"])],
                aggregation_rule: Some(
                    serde_json::json!({"clusterRoleSelectors":[{"matchLabels":{"rbac.example/workloads":"true"}}]}),
                ),
            },
        ],
        role_bindings: vec![
            binding(
                "RoleBinding",
                "audit-secrets",
                Some("prod"),
                vec![subject("Group", "platform", None)],
                "ClusterRole",
                "secret-reader",
            ),
            binding(
                "RoleBinding",
                "dev-pods",
                Some("dev"),
                vec![subject("User", "alice@example.com", None)],
                "Role",
                "dev-pods",
            ),
            binding(
                "RoleBinding",
                "authenticated-config",
                Some("prod"),
                vec![subject("Group", "system:authenticated", None)],
                "Role",
                "prod-config",
            ),
        ],
        cluster_role_bindings: vec![
            binding(
                "ClusterRoleBinding",
                "alice-observer",
                None,
                vec![subject("User", "alice@example.com", None)],
                "ClusterRole",
                "global-observer",
            ),
            binding(
                "ClusterRoleBinding",
                "alice-aggregated",
                None,
                vec![subject("User", "alice@example.com", None)],
                "ClusterRole",
                "aggregate-workloads",
            ),
        ],
    };

    let mut checks = vec![
        check("get", "", "secrets", Some("prod")),
        check("list", "", "secrets", Some("prod")),
        check("watch", "", "secrets", Some("prod")),
        check("get", "", "secrets", Some("staging")),
        check("get", "apps", "deployments", Some("staging")),
        check("list", "apps", "deployments", Some("prod")),
        check("watch", "apps", "deployments", Some("dev")),
        check("delete", "apps", "deployments", Some("prod")),
        check("get", "", "pods", Some("dev")),
        check("list", "", "pods", Some("dev")),
        check("create", "", "pods", Some("dev")),
    ];
    let mut exec_named = check("create", "", "pods", Some("dev"));
    exec_named.subresource = Some("exec".into());
    exec_named.resource_name = Some("toolbox".into());
    checks.push(exec_named.clone());
    let mut exec_other = exec_named.clone();
    exec_other.resource_name = Some("other".into());
    checks.push(exec_other);
    let mut exec_unnamed = exec_named;
    exec_unnamed.resource_name = None;
    checks.push(exec_unnamed);
    checks.extend([
        check("get", "", "configmaps", Some("prod")),
        check("get", "", "configmaps", Some("dev")),
        check("get", "", "namespaces", None),
        AccessCheck {
            verb: "get".into(),
            api_group: "".into(),
            resource: None,
            subresource: None,
            namespace: None,
            resource_name: None,
            field_selector: None,
            non_resource_url: Some("/healthz/ready".into()),
        },
        AccessCheck {
            verb: "post".into(),
            api_group: "".into(),
            resource: None,
            subresource: None,
            namespace: None,
            resource_name: None,
            field_selector: None,
            non_resource_url: Some("/healthz/ready".into()),
        },
        check("get", "batch", "jobs", Some("prod")),
    ]);

    let report = evaluate(
        &snapshot,
        &parse_subject("user:alice@example.com").unwrap(),
        &["platform".into()],
        &AccessMatrix { checks },
    );
    let decisions: Vec<bool> = report.checks.iter().map(|result| result.allowed).collect();
    assert_eq!(
        decisions,
        vec![
            true, true, false, false, true, true, true, false, true, true, false, true, false,
            false, true, false, true, true, false, true
        ]
    );
    assert_eq!(report.summary.total, 20);
    assert_eq!(report.summary.allowed, 12);
    assert_eq!(report.summary.denied, 8);
    assert_eq!(report.summary.uncertain, 1);
    assert_eq!(report.checks[0].grants[0].binding_name, "audit-secrets");
    assert!(report.checks[19].grants[0].uncertainty.is_some());
}

#[test]
fn service_account_default_namespace_and_groups_are_honored() {
    let snapshot = Snapshot {
        schema_version: "kpe.snapshot/v1".into(),
        collected_at: "now".into(),
        collector_version: "test".into(),
        context: "test".into(),
        server_version: "v1".into(),
        roles: vec![Role {
            kind: "Role".into(),
            metadata: metadata("read", Some("ops")),
            rules: vec![rule(&[""], &["pods"], &["get"])],
            aggregation_rule: None,
        }],
        cluster_roles: vec![],
        role_bindings: vec![binding(
            "RoleBinding",
            "service-accounts",
            Some("ops"),
            vec![subject("Group", "system:serviceaccounts:ops", None)],
            "Role",
            "read",
        )],
        cluster_role_bindings: vec![],
    };
    let report = evaluate(
        &snapshot,
        &parse_subject("serviceaccount:ops:bot").unwrap(),
        &[],
        &AccessMatrix {
            checks: vec![check("get", "", "pods", Some("ops"))],
        },
    );
    assert!(report.checks[0].allowed);
    assert_eq!(report.checks[0].grants[0].matched_subject.kind, "Group");
}

#[test]
fn resource_names_follow_kubernetes_verb_and_field_selector_semantics() {
    let mut named_rule = rule(
        &[""],
        &["pods", "pods/exec"],
        &["create", "deletecollection", "list", "watch", "get"],
    );
    named_rule.resource_names = vec!["approved".into()];
    let snapshot = Snapshot {
        schema_version: "kpe.snapshot/v1".into(),
        collected_at: "2026-08-28T00:00:00Z".into(),
        collector_version: "test".into(),
        context: "resource-name-regression".into(),
        server_version: "v1.33.4".into(),
        roles: vec![],
        cluster_roles: vec![Role {
            kind: "ClusterRole".into(),
            metadata: metadata("named-pods", None),
            rules: vec![named_rule],
            aggregation_rule: None,
        }],
        role_bindings: vec![],
        cluster_role_bindings: vec![binding(
            "ClusterRoleBinding",
            "alice-named-pods",
            None,
            vec![subject("User", "alice", None)],
            "ClusterRole",
            "named-pods",
        )],
    };

    let named = |verb: &str, selector: Option<&str>| {
        let mut request = check(verb, "", "pods", Some("payments"));
        request.resource_name = Some("approved".into());
        request.field_selector = selector.map(str::to_owned);
        request
    };
    let mut subresource_create = named("create", None);
    subresource_create.subresource = Some("exec".into());
    let report = evaluate(
        &snapshot,
        &parse_subject("user:alice").unwrap(),
        &[],
        &AccessMatrix {
            checks: vec![
                named("create", None),
                named("deletecollection", None),
                named("list", None),
                named("watch", Some("metadata.name=other")),
                named("list", Some("metadata.name=approved")),
                named("watch", Some("metadata.name==approved")),
                named("get", None),
                subresource_create,
            ],
        },
    );

    assert_eq!(
        report
            .checks
            .iter()
            .map(|result| result.allowed)
            .collect::<Vec<_>>(),
        vec![false, false, false, false, true, true, true, true]
    );
    assert!(
        report.checks[..4]
            .iter()
            .all(|check| check.grants.is_empty())
    );
    assert_eq!(report.summary.allowed, 4);
    assert_eq!(report.summary.denied, 4);
}

#[test]
fn field_selector_state_is_bounded_to_named_list_and_watch_checks() {
    let mut request = check("list", "", "pods", Some("payments"));
    request.field_selector = Some("metadata.name=approved".into());
    assert_eq!(
        request.validate().unwrap_err(),
        "fieldSelector requires resourceName"
    );

    request.resource_name = Some("approved".into());
    request.verb = "get".into();
    assert_eq!(
        request.validate().unwrap_err(),
        "fieldSelector is supported only for list or watch checks"
    );
}

#[test]
fn wildcard_subresource_rules_follow_kubernetes_resource_matching() {
    let snapshot = Snapshot {
        schema_version: "kpe.snapshot/v1".into(),
        collected_at: "2026-08-28T00:00:00Z".into(),
        collector_version: "test".into(),
        context: "wildcard-subresource-regression".into(),
        server_version: "v1.33.4".into(),
        roles: vec![],
        cluster_roles: vec![Role {
            kind: "ClusterRole".into(),
            metadata: metadata("scale-any-resource", None),
            rules: vec![rule(&["apps"], &["*/scale"], &["update"])],
            aggregation_rule: None,
        }],
        role_bindings: vec![],
        cluster_role_bindings: vec![binding(
            "ClusterRoleBinding",
            "alice-scale",
            None,
            vec![subject("User", "alice", None)],
            "ClusterRole",
            "scale-any-resource",
        )],
    };
    let with_subresource = |resource: &str, subresource: &str| {
        let mut request = check("update", "apps", resource, Some("payments"));
        request.subresource = Some(subresource.into());
        request
    };
    let report = evaluate(
        &snapshot,
        &parse_subject("user:alice").unwrap(),
        &[],
        &AccessMatrix {
            checks: vec![
                with_subresource("deployments", "scale"),
                with_subresource("statefulsets", "scale"),
                with_subresource("deployments", "status"),
                check("update", "apps", "deployments", Some("payments")),
            ],
        },
    );

    assert_eq!(
        report
            .checks
            .iter()
            .map(|result| result.allowed)
            .collect::<Vec<_>>(),
        vec![true, true, false, false]
    );
    assert_eq!(report.summary.allowed, 2);
    assert_eq!(report.summary.denied, 2);
    assert_eq!(report.checks[0].grants[0].rule.resources, vec!["*/scale"]);
}
