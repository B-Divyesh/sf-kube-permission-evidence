# Verify Kubernetes RBAC access evidence

**Verdict: PASS**

- Work order: `kube-permission-evidence-verify-5`
- Verified: 2026-09-05 UTC
- Findings: 0
- Untested public claims: 0
- Implementation reviewed: `a223caf74e6945d367ed361c9cc58fa42a24934d`
- Documentation reviewed: `66196611ffdb92af351dcee6b03599c90a60908e`
- Live URL: <https://kube-permission-evidence.sociobot.in/>

The implementation commit scopes non-resource URL evidence to
ClusterRoleBindings and keeps first-screen copy opaque during its motion. The
documentation commit follows it and changes only factory records.

## Job, audience, and first action

The job is to trace a Kubernetes access question to every binding, role, and
rule that grants it, then write an audit evidence packet. It is for Kubernetes
operators preparing audits or permission changes without changing cluster
state. Before scrolling, fresh 1440 x 900 and 390 x 844 browser contexts
showed the job, that audience, and **Try it with sample data**. The action was
at 666 px on desktop and 514 px on phone.

## Clean checkout and installed CLI

A detached clean worktree at `a223caf` was used. It stayed clean after the
checks.

```text
npm ci                                      PASS — 24 packages
npm audit --audit-level=high                PASS — 0 vulnerabilities
npm test                                    PASS — typecheck, formatting,
                                             Clippy, 19 Rust tests, 48 browser tests
npm run build                               PASS — release CLI and dist/site
cargo test --doc                            PASS — 1 doctest
cargo package --locked --allow-dirty        PASS
clean packaged cargo install                PASS — kpe 0.1.0
```

The installed binary provided its help and version, ran `kpe demo` without a
cluster or network, and created a Markdown and JSON packet with 4 checks: 2
allowed, 2 denied, and 0 uncertain. A missing snapshot reported a clear error
and exited 1. The installed-artifact claim also rechecked the repaired
non-resource URL path: a ClusterRoleBinding can grant `/healthz/*`, a
RoleBinding-only URL rule is denied, and a namespaced URL question is rejected.

## Public claims

Every one of the 34 commands declared in `.factory/claims.json` was run
separately from the clean worktree using its declared
`npm run test:claim -- --grep @claim:<id>` command. All passed.

| Claim group | Result |
| --- | --- |
| point-in-time-proof through json-stdout | 12 / 12 PASS |
| exit-codes through server-version-recorded | 12 / 12 PASS |
| limitations-recorded through sales-paused | 10 / 10 PASS |

There are no missing, duplicate, false, or untested declared claims. The
claims cover the bounded matrix, every causal grant path, read-only collection,
token-free snapshots, signing and trusted signer checks, role and binding
scope, wildcard and subresource behavior, `resourceNames`, isolated browser
demo state, first-party requests, offline behavior, license cache safety, and
the paused sales state.

## Live site

Fresh Chromium desktop and phone contexts checked the deployed site.

- `/`, `/demo/`, `/privacy/`, and `/terms/` returned 200. They each have the
  right title, one h1, `lang=en`, and a main landmark. The designed missing
  route returned HTTP 404 with **Page not found**; that deliberate 404 is
  expected, not a defect.
- The first action opened the populated demo in one click. It displayed the
  persistent **Demo — sample data, nothing is saved** label, an allowed sample
  with binding, role, and rule evidence, and a denied sample. Reset restored
  the first sample. Start for real cleared demo session state. A pre-seeded
  real license value was unchanged; while still in demo, all requests remained
  same-origin.
- Keyboard focus reached the skip link first. There was no mobile horizontal
  overflow. Reduced motion reduced the hero animation to less than 1 ms.
  After service-worker control, a phone context reloaded the demo offline and
  selected another sample successfully.
- Axe found zero serious or critical violations on root, demo, privacy, terms,
  and the designed 404. A default-motion probe paused the live hero at 200 ms:
  its text opacity was 1 and Axe still found zero serious or critical issues.
  Normal-route console and page errors were zero. The browser's console notice
  for the intentional 404 response was classified as expected.
- Every discovered HTTP link resolved successfully (12 checked). The sitemap
  lists root, demo, privacy, terms, and 404. Live headers include CSP with
  response-header `frame-ancestors`, HSTS, `nosniff`, strict referrer policy,
  and permissions policy.
- The 16 deployable public files from a fresh candidate build matched live
  byte-for-byte. Mobile Lighthouse was Performance 100, Accessibility 100,
  Best Practices 100, and SEO 100; LCP was 1.4 s, TBT 30 ms, and CLS 0.

## Earlier findings

All earlier review and verification findings are resolved and rechecked.

| Earlier issue | Current disposition |
| --- | --- |
| `resourceNames`, `*/subresource`, and non-resource URL false results | Resolved by installed-artifact, matrix, and individual claim tests. The RoleBinding URL false grant is specifically denied. |
| Rust format, TypeScript, headers, targets, and service-worker update gaps | Resolved by `npm test`, live header inspection, phone checks, and controlled offline reload. |
| Paid checkout and license cache concerns | Sales are plainly paused with no payment action. License token cache binding and cache-URL safety pass their individual claim test. |
| Unauthenticated added packet fields and absent trusted signer check | Resolved by strict packet and trusted-key regression tests, including the packaged CLI claim. |
| Missing demos, claims, plain words, 404, metadata, skeleton, and collector boundary | Resolved by the CLI and browser demos, all 34 claim commands, first-screen check, route/link checks, and the controlled collector claim. |
| Hero motion contrast | Resolved. At an explicit in-progress 200 ms state, hero copy remains fully opaque and has no serious or critical Axe issue. |

## Scope and remaining work

This is a static documentation site and local CLI. It has no product backend,
tenant store, health endpoint, or rate-limited live API, so backend-only tenant
isolation, restart persistence, health, and 429/Retry-After checks do not
apply. Cargo publication, deployment, billing registration, and DNS remain
factory-owned work. No product defect or untested public claim remains.
