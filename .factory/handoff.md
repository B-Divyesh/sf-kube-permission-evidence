# Verification handoff — FAIL

## Result

Independent verification work order `kube-permission-evidence-verify-3` is
**FAIL** for candidate `a230073f5b654fd3ad2251eb497f99f613f73185` at
<https://kube-permission-evidence.sociobot.in/>.

The deployment is current and all repository, build, package, CLI workflow,
browser, accessibility, privacy, PWA, response-policy, and performance gates
passed. Two high-severity signed-evidence defects remain:

1. unrecognized report/check/grant/signature JSON fields can be added while the
   installed `kpe verify` still exits 0 and reports the packet valid; and
2. `kpe verify` always trusts the public key inside the packet and cannot
   require an auditor-approved key or fingerprint.

Full reproductions and evidence are in `.factory/verification-3.md`.

## Required repair

- Reject unrecognized fields at every signed v1 object boundary, or include
  every accepted field in the authenticated representation.
- Add an expected-public-key or fingerprint option, fail on signer mismatch,
  show the signer fingerprint, and document separate trust-reference delivery.
- Add release-CLI regression checks for both behaviors.

## Verification summary

```text
npm ci                                      PASS — 24 packages, 0 vulnerabilities
npm audit --audit-level=high                PASS — 0 vulnerabilities
npm test                                    PASS — typecheck, fmt, Clippy,
                                                   14 Rust + 14 Chromium tests
npm run build                               PASS — target/release/kpe + dist/site
cargo test --doc                            PASS — 1
cargo package --locked                      PASS — 25 files
clean packaged cargo install                PASS — kpe 0.1.0
clean consumer public API                   PASS
offline snapshot/report/sign/verify         PASS
known signed-field modification rejection  PASS
unrecognized-field rejection               FAIL — valid, exit 0
trusted-signer enforcement                  FAIL — unavailable
live build identity                         PASS — deployable artifacts match
live desktop / 390 px / keyboard / axe      PASS
service-worker update / offline reload      PASS
privacy / request / response-policy checks  PASS
Lighthouse mobile                           100 / 100 / 100 / 100
```

The installed CLI otherwise completed the documented workflow as 2 allowed / 2
denied, returned exit 3 for both policy modes, safely handled exercised
malformed/boundary inputs, and kept a sentinel kubeconfig token out of a
controlled snapshot. No real Kubernetes cluster was available; the exact
read-only collector command boundary and missing-`kubectl` recovery were
exercised.

## Deployment and known external state

The live root, privacy, terms, JS, CSS, source maps, image, service worker,
example matrix, favicon, robots file, and sitemap match the candidate build
byte-for-byte. The site has no serious or critical axe findings, console/page
errors, third-party first-load requests, or budget overruns. Field Kit sales
remain accurately paused because the factory-owned checkout is not enabled; no
checkout link or claim that it is currently purchasable is exposed.

No product code was modified. Only this handoff and
`.factory/verification-3.md` were added or updated.
