# Kube Permission Evidence

`kpe` creates point-in-time evidence for Kubernetes RBAC access. It is for
operators preparing audits or reviewing permission changes.

The CLI collects four RBAC object types, evaluates named access questions, and
writes every matching binding and rule. It does not change cluster state,
install an agent, send telemetry, or use a hosted cluster connection.

## Try the bundled sample

The sample uses fictional data built into the binary. It needs no kubeconfig,
kubectl, or network connection.

```sh
cargo run -- demo
```

The command creates a new temporary directory and prints its path. The
directory contains the sample snapshot, matrix, Markdown packet, and JSON
packet. Use an explicit new directory when needed:

```sh
cargo run -- demo --output /tmp/kpe-sample
```

The browser version is at
<https://kube-permission-evidence.sociobot.in/demo/>. Its state uses the
`demo:kpe:` session-storage prefix and never reads or changes real product
data. See [the demo contract](.factory/demo.md).

## Install

Build the single Rust binary from this checkout:

```sh
cargo install --path .
kpe --help
```

The package starts at version `0.1.0`. The factory publishes releases; this
repository does not use registry credentials.

Live collection needs kubectl and read-only RBAC access. Offline reports need
neither kubectl nor a cluster.

## Create an evidence packet

### 1. Write bounded questions

Create `matrix.json`:

```json
{
  "checks": [
    {"verb":"get", "apiGroup":"", "resource":"secrets", "namespace":"payments"},
    {"verb":"list", "apiGroup":"apps", "resource":"deployments", "namespace":"payments"},
    {"verb":"create", "apiGroup":"", "resource":"pods", "subresource":"exec", "namespace":"payments"}
  ]
}
```

Unknown matrix fields are rejected. This prevents a misspelling from changing
the access question.

For named `get`, `update`, `patch`, `delete`, or subresource requests, add
`"resourceName":"api-key"`. A named `list` or `watch` check also needs the
field selector sent by the client:

```json
{"verb":"list", "resource":"secrets", "namespace":"payments", "resourceName":"api-key", "fieldSelector":"metadata.name=api-key"}
```

Kubernetes cannot restrict top-level `create` or `deletecollection` requests
by name. A `resourceNames` rule does not grant those checks.

For a non-resource endpoint, use this shape:

```json
{"verb":"get", "nonResourceURL":"/healthz/ready"}
```

Non-resource questions cannot include `apiGroup`, `namespace`, `subresource`,
`resourceName`, or `fieldSelector`. Kubernetes applies non-resource URL rules
only through a ClusterRoleBinding; a RoleBinding never grants one.

### 2. Collect and evaluate

```sh
kpe report \
  --subject user:alice@example.com \
  --as-group platform-engineers \
  --matrix matrix.json \
  --output evidence
```

This writes `evidence.md` and `evidence.json`. `--subject` accepts
`user:<name>`, `group:<name>`, or
`serviceaccount:<namespace>:<name>`. Standard Kubernetes service-account
groups are derived automatically. Add external identity-provider groups with
repeated `--as-group` options.

The report exits `0` after a completed evaluation. Input, collection, and
output errors exit `1`. `--fail-on denied` or `--fail-on allowed` exits `3`
when the selected result occurs.

Use `--json` to send the complete JSON packet to stdout without creating
files. Diagnostics remain on stderr.

### 3. Collect once and report offline

```sh
kpe snapshot --output rbac-snapshot.json --context audit-readonly
kpe report --snapshot rbac-snapshot.json \
  --subject user:alice@example.com \
  --as-group platform-engineers \
  --matrix matrix.json \
  --output evidence
```

Snapshots contain RBAC objects and collection metadata. They do not contain
kubeconfig credentials or bearer tokens.

### 4. Sign and verify with a trusted signer

```sh
kpe keygen \
  --output audit-signing.key \
  --public-key-output audit-signing.pub

kpe report --snapshot rbac-snapshot.json \
  --subject user:alice@example.com \
  --matrix matrix.json \
  --output evidence \
  --signing-key audit-signing.key

kpe verify evidence.json --trusted-public-key audit-signing.pub
```

`keygen` creates an owner-only private key on Unix and refuses to overwrite an
existing file. Deliver `audit-signing.pub` to the auditor through a separate,
trusted channel. The command also prints its SHA-256 fingerprint, which can be
checked directly:

```sh
kpe verify evidence.json \
  --trusted-fingerprint SHA256:<64-hex-character-fingerprint>
```

Verification checks the content digest, Ed25519 signature, strict packet
schema, and the optional trusted signer. It prints the
packet signer fingerprint. A different trusted key or any added packet field
causes a nonzero result.

Without a trust option, verification reports content integrity only and says
that signer trust was not checked.

## RBAC behavior covered

- RoleBinding to Role grants.
- RoleBinding to ClusterRole grants within the binding namespace.
- ClusterRoleBinding to ClusterRole grants across namespaces.
- Exact and wildcard verbs, API groups, resources, and non-resource URLs.
- Exact subresources and Kubernetes `*/subresource` rules.
- Verb-correct `resourceNames` and named list or watch field selectors.
- User, Group, and ServiceAccount subjects.
- Every matching grant path, rather than only the first.

Aggregated ClusterRole rules come from the controller-resolved API response.
Each affected result is marked uncertain, and the packet records the
Kubernetes server version.

Webhook authorizers, Node authorizers, impersonation, and unsupplied external
group membership remain outside the snapshot. These limits are written into
every packet.

## Read-only collection boundary

The collector can invoke these commands. Cluster calls append context and
kubeconfig flags when supplied. The client-version call does not need them.

```sh
kubectl version --client -o json
kubectl get roles --all-namespaces -o json
kubectl get rolebindings --all-namespaces -o json
kubectl get clusterroles -o json
kubectl get clusterrolebindings -o json
kubectl config current-context
kubectl version -o json
```

`kubectl config current-context` runs only when `--context` is omitted. The
collector never invokes create, apply, patch, replace, delete, edit, or exec.

## Privacy and payment state

The CLI has no network client and sends no telemetry. The static site uses no
analytics, trackers, CDN fonts, or third-party scripts. Its guide and demo work
offline after the first visit.

Existing Field Kit licenses are stored in browser local storage and verified
through the Sociobot API at most once daily. License return URLs are excluded
from Cache Storage. Field Kit sales are paused, and no payment action is
available.

See the live [privacy policy](https://kube-permission-evidence.sociobot.in/privacy/)
and [terms](https://kube-permission-evidence.sociobot.in/terms/).

## Development and verification

```sh
npm ci
npm test
npm run build
cargo test --doc
cargo package --locked --allow-dirty
```

`npm test` runs TypeScript, formatting, Clippy, Rust behavior, all 34 declared
claim checks, browser flows, accessibility checks, privacy checks, and offline
checks. Every public claim and its isolated command is listed in
[`.factory/claims.json`](.factory/claims.json).

`npm run build` creates the release binary at `target/release/kpe` and the
static site at `dist/site/`. To exercise the installable artifact, unpack the
crate and install it into a clean Cargo root:

```sh
cargo package --locked --allow-dirty
cargo install --locked \
  --path target/package/kube-permission-evidence-0.1.0 \
  --root /tmp/kpe-consumer
```

## Deployment

The factory deploys `dist/site/` to
<https://kube-permission-evidence.sociobot.in>. It owns DNS, release
publishing, and billing registration.

## License

MIT. See [LICENSE](LICENSE). Changes are recorded in
[CHANGELOG.md](CHANGELOG.md).
