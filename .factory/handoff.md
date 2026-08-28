# Handoff — Kube Permission Evidence v0.1.0

## What shipped

- A Rust `kpe` single binary with four focused commands: `snapshot`, `report`,
  `keygen`, and `verify`.
- Read-only live collection through five documented kubectl calls. KPE passes a
  requested context/kubeconfig to kubectl but never parses or stores its token.
- Offline snapshot evaluation for User, Group, and ServiceAccount subjects;
  Kubernetes service-account groups are derived and external groups are
  explicit inputs.
- Exact evaluation of namespaced RoleBindings, ClusterRoleBindings, Role and
  ClusterRole refs, wildcards, subresources, `resourceNames`, and non-resource
  URL suffix wildcards.
- Complete causal grants in both Markdown and JSON, with SHA-256 source
  fingerprinting and explicit caveats. Aggregated ClusterRole results are
  marked uncertain and linked to the captured Kubernetes server version.
- Optional local Ed25519 key generation, evidence signing, tamper detection,
  and offline signature verification.
- A production static site at `dist/site/` with install guidance, a keyboard-
  accessible recorded evidence walkthrough, mobile layout, empty/offline/error
  states, privacy and terms pages, and a service-worker cache.
- One-time $49 Field Kit purchase and restore flow using only the Sociobot
  billing API. The cached verdict is optimistic at first paint and refreshed
  no more than daily. Core evaluation, export, signing, accessibility, and
  safety remain free.
- Original botanical field-guide hero generated with the `factory-image`
  deployment and optimized from PNG to a 92 KB WebP. The exact prompt and
  provenance are in `.factory/design.md` and `.factory/specimen-map.prompt.json`.

## Run and verify

```sh
npm install
npm test
npm run build
cargo package --allow-dirty
```

`npm test` runs 8 Rust tests (including the documented end-to-end signed packet
flow and a 20-case RBAC fixture) plus 6 Playwright checks across desktop,
390 px mobile, license return, offline behavior, privacy, and terms. Axe reports
no serious or critical violations.

The exact deployment build command is `npm run build`; the static artifact is
`dist/site/index.html`. The release binary is `target/release/kpe`. The verified
source package is `target/package/kube-permission-evidence-0.1.0.crate`; the
factory can reproduce it with `cargo package` from a clean tree and owns
publishing credentials.

Manual CLI smoke test used:

```sh
kpe keygen --output /tmp/audit.key
kpe report --snapshot examples/rbac-snapshot.json \
  --subject user:alice@example.com --as-group platform-engineers \
  --matrix examples/matrix.json --output /tmp/evidence \
  --signing-key /tmp/audit.key
kpe verify /tmp/evidence.json
```

Result: 2 allowed, 2 denied; signature valid; signing key mode `0600`.

## Quality evidence

- Lighthouse 13.4.1, production build, simulated mobile: Performance 100,
  Accessibility 100, Best Practices 100, SEO 100.
- FCP 1.0 s; LCP 1.6 s; TBT 0 ms; CLS 0.
- Initial authored JS: 5.95 KB raw / 2.68 KB gzip; CSS: 14.09 KB raw /
  4.08 KB gzip; hero: 93,322 bytes. No runtime CDN or webfont requests.
- Factory `verify-url.sh`: HTTP 200, 535 ms network-idle load, title/lang/main
  present, exactly one h1, zero images missing alt, zero unlabeled buttons, and
  zero browser console errors.
- `npm audit --audit-level=high`: 0 vulnerabilities.
- `cargo clippy --all-targets -- -D warnings`: passed.

## Known gaps and next steps

- No real Kubernetes API was available in the build container, so live
  collection was not exercised against a cluster. Collection is intentionally
  thin kubectl orchestration; offline fixtures exercise the evaluator and all
  report paths.
- KPE proves RBAC grants only. Webhook, Node, ABAC, and external IdP membership
  cannot be inferred from these four object families and are stated as
  limitations in every packet.
- The factory must register the paid product and attach maintained release
  binaries/templates before the Field Kit fulfillment panel can expose those
  downloads. The production checkout and verification URLs already follow the
  required slug-based contract; no product ID is hardcoded.
- Registry and GitHub release publishing were intentionally not performed.
