# Verify Kubernetes RBAC evidence packets — PASS

**Work order:** `kube-permission-evidence-verify-4`  
**Verdict:** **PASS**  
**Findings:** 0  
**Untested public claims:** 0  
**Implementation reviewed:** `98eb121448edd957f8494df9577b7c47dcbc158b`  
**Documentation reviewed:** `abe513f8391b92fc94f24d28b9891dab0257594d`  
**Live URL:** <https://kube-permission-evidence.sociobot.in/>  
**Verified:** 2026-09-05 UTC

## Job, audience, and first action

The job is to trace a Kubernetes access question to every binding, role, and
rule that grants it, then write an auditor-readable evidence packet. It is for
Kubernetes operators preparing audits or permission changes. Before scrolling,
fresh desktop and 390 px phone contexts showed the headline **“Trace Kubernetes
access to every granting rule”**, named Kubernetes operators, and showed
**“Try it with sample data”** as the first action. Its adjacent text says the
sample opens four checks and never contacts a cluster.

## Decision

**PASS.** The implementation and live site complete the researched read-only
RBAC-evidence job. There are no high, medium, low, or minor findings, and every
declared public claim was executed and passed from a fresh detached checkout.

The deployment matches a fresh build of implementation commit `98eb121`.
Documentation commit `abe513f` only records the preceding repair handoff; it
does not change deployable product files.

## Clean checkout and installed artifact

A fresh detached worktree at `98eb121448edd957f8494df9577b7c47dcbc158b` was
clean before and after verification. These commands passed:

```text
npm ci                                      PASS — 0 vulnerabilities
npm audit --audit-level=high                PASS — 0 vulnerabilities
npm test                                    PASS — typecheck, format, Clippy,
                                             18 Rust tests, 47 browser tests
npm run build                               PASS — release CLI and dist/site
cargo test --doc                            PASS — 1 doctest
cargo package --locked --allow-dirty        PASS — 25 files, package verified
```

The packaged crate was installed into a new empty Cargo root. The installed
`kpe 0.1.0` binary provided helpful `--help`, created the bundled four-check
demo (2 allowed, 2 denied), wrote Markdown and JSON packets, and completed a
signed report verification with its matching trusted public key. A different
trusted public key failed with a clear nonzero signer-mismatch error. Boundary
and recovery checks also passed: an empty matrix returns a valid 0/0/0 packet,
an invalid subject returns nonzero, and snapshot collection with no `kubectl`
returns nonzero without writing a snapshot.

## Public claims

All 34 commands in `.factory/claims.json` were invoked separately with their
declared `npm run test:claim -- --grep @claim:<id>` command. Each ran its own
clean browser/CLI sandbox and passed. There are no missing, duplicate, false,
or untested claim entries.

| Claim IDs verified independently | Result |
| --- | --- |
| point-in-time-proof; collector-command-boundary; bounded-matrix; all-grant-paths; read-only-collection; no-kubeconfig-token; single-binary-no-agent; no-telemetry; no-hosted-cluster; offline-report; markdown-json-output; json-stdout | PASS |
| exit-codes; subject-forms; service-account-groups; rolebinding-role; rolebinding-clusterrole-namespace; clusterrolebinding-clusterwide; wildcard-matching; subresource-matching; resource-name-semantics; non-resource-url; aggregation-uncertain; server-version-recorded | PASS |
| limitations-recorded; token-free-snapshot; key-safety; signature-trust; least-privilege-collection; isolated-browser-demo; first-party-site; offline-site; license-storage; sales-paused | PASS |

The checks cover the earlier security regressions directly: strict rejection of
unknown signed-packet and matrix fields; matching and mismatching trusted
signers; `resourceNames` verb and selector behavior; and Kubernetes
`*/subresource` matching. They also cover the documented read-only kubectl
boundary, no token snapshot, all grant paths, signing keys, demo isolation,
privacy requests, offline reload, license cache binding, and paused sales.

## Live site

Fresh Playwright contexts checked the published desktop and phone pages.

- `/`, `/demo/`, `/privacy/`, and `/terms/` return 200 with their required
  route titles, one h1, `lang=en`, main landmark, labeled controls, and no
  console or page errors.
- `/missing-verify4` returns the designed **Page not found** route with HTTP
  404. This deliberate 404 is expected and is not a defect.
- The root action opens `/demo/` in one click. The populated first sample is
  allowed and explains its binding, role, and rule. The persistent **Demo —
  sample data, nothing is saved** label remains visible. Reset restores the
  first check, and Start for real clears the `demo:kpe:` session state. A
  real-data sentinel stayed unchanged throughout demo use.
- A clean normal load requested only the product origin. The license verifier
  is separately documented to contact Sociobot only when an existing license
  token is present. No analytics, tracker, CDN font, or third-party script was
  observed in the normal/demo flow.
- Keyboard Tab reaches the skip link first and its designed focus outline is
  3 px. Demo selection, reset, copy fallback, and links are keyboard usable.
  At 390 px there is no horizontal overflow or visible interactive target
  under 44 px. Reduced motion reports a `0.01s` transition.
- After service-worker control, a fresh 390 px context was taken offline,
  reloaded `/demo/`, saw the offline notice, and selected another bundled
  permission question successfully.
- Playwright axe reported zero serious or critical violations on root, demo,
  privacy, terms, and the 404 route. The factory URL verifier also passed on
  root, demo, privacy, and terms.
- Internal pages, sample download, legal links, and public source links
  resolved. The missing route's own skip link correctly resolves to its
  expected 404 response.

The live headers include CSP, HSTS, `nosniff`, strict referrer policy, and a
permissions policy. The live root and all generated route pages, service
worker, robots file, sitemap, icons, illustration, JS, CSS, and example matrix
matched the fresh implementation build byte-for-byte.

Live mobile Lighthouse retry: Performance 100, Accessibility 100, Best
Practices 100, SEO 100; LCP 1.38 s, CLS 0, and TBT 13 ms.

## Earlier findings

All prior review and verification findings are resolved and were rechecked:

| Earlier finding | Current disposition |
| --- | --- |
| `resourceNames` false grants | Resolved by regression and independent installed-binary checks. |
| Formatting and TypeScript quality gates | Resolved; both pass through `npm test`. |
| Cache/security policy and mobile target gaps | Resolved in live headers and fresh phone checks. |
| Service-worker update/offline behavior | Resolved; controlled offline reload and selection pass. |
| `*/subresource` false denial | Resolved by regression and claim test. |
| Paid checkout unavailable | Not a current product claim: sales are plainly paused and no payment control is shown. |
| License cache token binding and cache URL privacy | Resolved by claim test and live offline/privacy flow. |
| Unknown signed JSON fields | Resolved by strict packet regression and installed artifact test. |
| Trusted signer requirement | Resolved by matching/mismatching trusted-key installed tests. |
| Missing CLI/browser demo, claims, plain words, 404, metadata, skeleton, and collector documentation | Resolved by the current CLI demo, browser flow, all 34 claim commands, route checks, and first-screen inspection. |

## Evidence and scope

Evidence is under `/work/.evidence/verify4-live/`, including fresh desktop and
phone screenshots, browser assertions, axe output, route/link results,
artifact comparisons, factory URL-verifier output, and Lighthouse JSON.

This product is a static documentation site plus a local CLI. It has no product
backend, tenant store, health endpoint, or rate-limited live API to test. The
backend-only tenant isolation, restart persistence, and 429/Retry-After checks
do not apply. Cargo registry publication remains a factory release task; no
registry credential was used.
