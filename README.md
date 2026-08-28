# Kube Permission Evidence

`kpe` turns Kubernetes RBAC into an auditor-readable, point-in-time proof. It
collects only Roles, ClusterRoles, RoleBindings, and ClusterRoleBindings,
evaluates a bounded set of questions for one subject, and emits Markdown plus
machine-readable JSON showing every rule and binding that grants each access.

It is for Kubernetes operators preparing an audit or reviewing a permission
change without mutating the cluster. There is no admission controller, agent,
telemetry, or hosted cluster connection.

## Install

Build the single Rust binary:

```sh
cargo install --path .
kpe --help
```

`kpe` is versioned from `0.1.0`. A packaged source release can be inspected
with `cargo package --allow-dirty` (the factory publishes releases; this
repository never uses registry credentials).

Live collection needs `kubectl` and a kubeconfig that can only `get` and
`list` RBAC objects. Offline reports need neither a cluster nor `kubectl`.

## Usage

### 1. Write the questions you need to prove

`matrix.json`:

```json
{
  "checks": [
    {"verb":"get", "apiGroup":"", "resource":"secrets", "namespace":"payments"},
    {"verb":"list", "apiGroup":"apps", "resource":"deployments", "namespace":"payments"},
    {"verb":"create", "apiGroup":"", "resource":"pods", "subresource":"exec", "namespace":"payments"}
  ]
}
```

Every field is explicit. For a named object, add `"resourceName":"api-key"`.
For a non-resource endpoint, use `{"verb":"get","nonResourceURL":"/healthz"}`
instead of resource fields.

### 2. Create the evidence packet

```sh
kpe report \
  --subject user:alice@example.com \
  --as-group platform-engineers \
  --matrix matrix.json \
  --output evidence
```

This writes `evidence.md` and `evidence.json`. `--subject` accepts
`user:<name>`, `group:<name>`, or `serviceaccount:<namespace>:<name>`.
Known Kubernetes groups for service accounts are derived automatically;
supplement user identity-provider groups with repeated `--as-group` options.

By default, `kpe` uses the current kubectl context. Pin one if needed:

```sh
kpe report --subject serviceaccount:payments:reconciler \
  --matrix matrix.json --context audit-readonly --output reconciler-access
```

The report exits `0` when evaluation completed, whether an individual check is
allowed or denied. Collection, input, and output failures exit non-zero. Add
`--fail-on denied` to make any denied check exit `3`, or `--fail-on allowed`
to flag unexpected grants in CI.

### 3. Collect once, evaluate offline

```sh
kpe snapshot --output rbac-snapshot.json --context audit-readonly
kpe report --snapshot rbac-snapshot.json --subject user:alice@example.com \
  --as-group platform-engineers --matrix matrix.json --output evidence
```

The snapshot contains RBAC objects and collection metadata, never kubeconfig
credentials or bearer tokens.

### 4. Sign and verify a packet

```sh
kpe keygen --output audit-signing.key
kpe report --snapshot rbac-snapshot.json --subject user:alice@example.com \
  --matrix matrix.json --output evidence --signing-key audit-signing.key
kpe verify evidence.json
```

`keygen` writes a local Ed25519 seed with owner-only permissions on Unix. The
JSON packet embeds its public key, signature, and SHA-256 content digest;
`verify` recomputes all three without cluster access. Protect the key like any
other audit signing credential and never commit it.

### JSON output for scripts

`kpe report ... --json` writes the complete JSON report to stdout and does not
write files. Diagnostics go to stderr. `kpe snapshot --json` likewise streams
the snapshot.

## What the evaluator covers

- RoleBinding→Role and RoleBinding→ClusterRole grants, constrained to the
  binding namespace.
- ClusterRoleBinding→ClusterRole grants across namespaces.
- Exact and wildcard verbs, API groups, resources, and non-resource URLs.
- Subresources (`pods/exec`) and `resourceNames` constraints.
- Direct User, Group, and ServiceAccount subjects, including Kubernetes'
  standard service-account groups.
- Every granting path, not only the first match.

ClusterRoles with `aggregationRule` are evaluated from the controller-resolved
`rules` returned by the API and every affected proof is marked **uncertain**.
The report records Kubernetes server version and explains this version-sensitive
behavior. Impersonation, webhook authorizers, Node authorizers, and external
identity-provider group expansion are outside an RBAC snapshot, so `kpe` does
not claim to model them.

## Required read-only Kubernetes access

The collector runs only these calls:

```sh
kubectl get roles --all-namespaces -o json
kubectl get rolebindings --all-namespaces -o json
kubectl get clusterroles -o json
kubectl get clusterrolebindings -o json
kubectl version -o json
```

Use a dedicated audit identity. `kpe` never invokes create, patch, apply,
delete, exec, or an authorization mutation.

## Development

```sh
npm install
npm test
npm run build         # Rust release binary + site at dist/site/
npm run build:site    # static site only at dist/site/
cargo package --allow-dirty
```

The static documentation uses Vite and vanilla TypeScript. It has no runtime
CDNs, tracking, or uploaded cluster data. The license form stores only the
license token and a daily verification verdict in local browser storage.

## Deployment

The factory deploys `dist/site/` to
<https://kube-permission-evidence.sociobot.in>. It owns DNS, billing product
registration, binary release publishing, and registry credentials.

## License

MIT. See [LICENSE](LICENSE). Changes are recorded in [CHANGELOG.md](CHANGELOG.md).
