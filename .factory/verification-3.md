# Independent verification 3 — FAIL

**Work order:** `kube-permission-evidence-verify-3`
**Candidate commit:** `a230073f5b654fd3ad2251eb497f99f613f73185`
**Live URL:** <https://kube-permission-evidence.sociobot.in/>
**Verified:** 2026-08-28 UTC
**Earlier reports:** `.factory/verification.md`, `.factory/verification-2.md`

## Decision

**FAIL.** The current deployment matches the candidate's deployable artifacts,
and the repository, package, CLI workflow, browser, accessibility, privacy,
service-worker, response-policy, and performance checks pass. Fresh packaged
CLI verification found two **high-severity signed-evidence defects**:

1. unrecognized JSON fields are omitted from the authenticated representation,
   yet `kpe verify` reports the modified packet as valid; and
2. verification trusts the public key embedded in each packet and provides no
   way to require an auditor-approved signing key.

Signed, auditor-readable evidence is central to the researched job-to-be-done.
These are release-blocking product defects, not deployment-only failures.

## Defects

### High — unrecognized JSON fields remain unauthenticated while verification returns valid

`kpe verify` deserializes the input directly into `EvidenceReport`
(`src/main.rs:211-213`). The report and its nested signed types do not use
Serde's `deny_unknown_fields` (`src/model.rs:182-249`). Verification then
reserializes only the recognized typed fields before checking the digest and
Ed25519 signature (`src/packet.rs:70-98`). Added fields disappear before the
cryptographic check.

Fresh reproduction used `kpe 0.1.0` installed from the packaged candidate:

```sh
kpe keygen --output audit.key
kpe report --snapshot examples/rbac-snapshot.json \
  --subject user:alice@example.com --as-group platform-engineers \
  --matrix examples/matrix.json --output evidence --signing-key audit.key

jq '.checks[2].displayVerdict="ALLOWED"' \
  evidence.json > modified-check-level.json
kpe verify modified-check-level.json --json
```

The check still contains the recognized value `"allowed": false`, but the
additional display claim is present. Actual result:

```text
{"valid":true,"subject":"user:alice@example.com"}
exit 0
```

Separate additions at the report, check, grant, and signature object levels
all returned the same valid result with exit 0. The files were different:

```text
354b4c55fbad1c1dfeb188a40b81d8efc3f962ef08778cf90a37f018dbd9f1f2  evidence.json
f0b7bd77372006a54595505c7e684ffd54720a2c96a44a63c9b5634fcaeb3bc9  modified-check-level.json
```

As a control, changing recognized field `.summary.total` to `99` exited 1 with
`content digest does not match; packet was changed`.

This permits additional claims in the JSON object, individual checks, grant
proofs, or signature metadata while the official verifier authenticates only
the recognized subset. A human reviewer or a schema-extending consumer can
therefore see content that the verifier did not authenticate.

Required remediation: make the signed v1 schema strict at every nested object
boundary, or authenticate every field in the accepted JSON representation.
Add release-CLI checks proving that unrecognized report-, check-, grant-, and
signature-level fields return nonzero; retain the recognized-field control.

### High — verification has no trusted-signer check

The verifier always obtains `publicKey` from the packet itself
(`src/packet.rs:85-98`). `kpe verify --help` accepts only the packet path and
`--json`; it has no expected-key, fingerprint, keyring, or trust-policy option.

As a fresh control using the packaged CLI, a second report was created for
`reviewed@example.com` and signed with a separately generated key. Its public
key fingerprint differed from the first packet. The installed verifier gave
both independently signed files the same successful status:

```text
VALID — Ed25519 signature and SHA-256 digest match for user:alice@example.com.
exit 0
VALID — Ed25519 signature and SHA-256 digest match for user:reviewed@example.com.
exit 0
```

The signature mathematics are valid in both cases, but the documented command
establishes only packet self-consistency. It cannot confirm that the key belongs
to the operator whose evidence the auditor intended to trust. This falls short
of a signed audit-evidence workflow and makes private-key protection, which the
README correctly recommends, insufficient on its own.

Required remediation: let verification require an expected public key or
fingerprint, return nonzero on mismatch, display an unambiguous signer
fingerprint, and document how operators deliver that trust reference separately
from the packet. Add a regression check for a valid signature from a different
key.

## Clean checkout and repository gates

All repository commands ran in a newly created detached worktree at the exact
candidate. The worktree remained clean.

```text
npm ci                                      PASS — 24 packages, 0 vulnerabilities
npm audit --audit-level=high                PASS — 0 vulnerabilities
npm test                                    PASS
  npm run typecheck                         PASS
  cargo fmt --all -- --check                PASS
  cargo clippy --all-targets -- -D warnings PASS
  Rust unit/integration tests               PASS — 14
  Chromium site tests                       PASS — 14
npm run build                               PASS — release CLI + dist/site
cargo test --doc                            PASS — 1
cargo package --locked                      PASS — 25 files
```

The seeded 20-case matrix passed with the expected 12 allowed, 8 denied, and 1
uncertain decisions. Regression cases for top-level `resourceNames`, named
list/watch selectors, and Kubernetes `*/subresource` matching also passed.

## Package, public API, and installed CLI

The `.crate` was unpacked outside the repository and installed into a fresh
consumer root:

```sh
cargo install --locked --path <unpacked-crate> --root <clean-consumer-root>
```

Package metadata records candidate SHA
`a230073f5b654fd3ad2251eb497f99f613f73185`. The installed binary reports
`kpe 0.1.0` and provides helpful, non-interactive `snapshot`, `report`,
`keygen`, and `verify` commands. A separate Rust consumer compiled and ran the
documented public `parse_subject`/`evaluate` API, producing 0 allowed / 1
denied as expected.

Fresh end-to-end CLI evidence:

- Generated a 32-byte Ed25519 seed in a mode-`0600` file.
- Evaluated the packaged example as 2 allowed / 2 denied, with one causal grant
  for each allowed result plus source version and snapshot digest.
- Wrote readable Markdown and structured JSON companions.
- Verified the unchanged signed packet with exit 0.
- Returned exit 3 for both exercised `--fail-on denied` and `--fail-on allowed`
  conditions while retaining JSON output.
- Accepted an empty matrix and emitted a valid 0/0/0 report.
- Returned actionable nonzero errors for an empty subject, malformed JSON,
  blank verb, conflicting resource/non-resource target, unbound field
  selector, omitted output mode, conflicting output modes, missing `kubectl`,
  key overwrite, and an unsigned packet.

A controlled `kubectl` harness confirmed the collector invokes only client
version, `get roles`, `get rolebindings`, `get clusterroles`, `get
clusterrolebindings`, and server version commands. Context and kubeconfig path
were passed through, and a sentinel kubeconfig token did not appear in the
snapshot. No real Kubernetes cluster was available, so live-cluster collection
was not repeated; the missing-`kubectl` recovery path also passed.

## Exact production artifacts and budgets

`npm run build` produced the release binary and `dist/site/`:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| source crate | 37,983 | `7cb2e112c6fd84a044cdce30ea68ac8c44707477583baaa0722ab9263b57f147` |
| release `kpe` | 1,591,264 | `a3ea03572828b46e4044797f4ad5a9e56db53c00d59f5b01e3888f2c0fad40f8` |
| landing HTML | 10,678 | `24b615a12c1569623473540c247c1996d087feaadf849867b5bfe248d548f8f7` |
| main JS | 5,986 | `b99f639c4827b293e740d1357e12abccf58b14aad88d6fb2631f34d5be916e6a` |
| CSS | 14,183 | `8619e6173c6e700645b985320f2663551eef76a524e313ea5dfc94ce1de9dbfc` |
| hero WebP | 93,322 | `b71aca102454a1d83c31d17969dceb8013f047edae8bc5e6a90990b23f0c12eb` |
| service worker | 1,303 | `f7afc179252beee98b4dae126729b24fa45413d24bf677dce816d2dda6b84c94` |

Main JavaScript is 2,714 bytes gzip, CSS is 4,085 bytes gzip, there are no font
files, and the hero is 93,322 bytes. All are within the supplied 200 KB JS, 50
KB CSS, 120 KB font, and 300 KB hero budgets.

## Live deployment and build identity

The live root, privacy page, terms page, hashed JS/CSS and source maps, hero
image, service worker, example matrix, favicon, robots file, and sitemap
matched the fresh candidate build byte-for-byte. Candidate `a230073f` differs
from deployed source repair `aed9592` only in `.factory/handoff.md`; the live
`Last-Modified` value is 2026-08-28 05:19:01 UTC, after the candidate commit.
The deployment is current for every generated public artifact.

- `/`, `/privacy/`, and `/terms/` return HTTPS 200; HTTP redirects with 301.
- HTML uses 30-second revalidation; hashed JS/CSS and the hero use one-year
  immutable caching; conditional requests returned 304.
- `sw.js` uses `no-cache, no-store, must-revalidate`.
- Responses include CSP, Permissions-Policy, HSTS, strict referrer policy, and
  `nosniff`.
- The live license verifier returned `valid:false`, `reason:"invalid"`,
  origin-specific CORS, and `Cache-Control: no-store` for a QA-invalid token.
- The factory URL verifier passed in 827 ms with title, `lang=en`, one h1, one
  main, complete image alt text, labeled buttons, and no console/page errors.

## Browser, accessibility, privacy, and PWA evidence

Fresh Chromium at 1440×900 and 390×844 covered `/`, `/privacy/`, and `/terms/`:

- zero serious or critical axe findings and zero console/page errors;
- one h1 and main landmark per page, `lang=en`, complete alt text, 16 px body
  text, no horizontal overflow, and no visible target below 44×44 px;
- keyboard-only skip link, demo select, clear button, and copy control work;
- the focused skip link has a visible 3 px clay outline;
- allowed, denied, uncertain, empty, copy-success, and copy-unavailable recovery
  states were exercised;
- reduced motion changed animation/transition duration to 0.01 ms and smooth
  scrolling to `auto`;
- a clean first load created no local/session storage and contacted only the
  product origin; no analytics, tracker, CDN font, or third-party script was
  observed;
- replacement license tokens were each verified, stripped from the visible
  URL, bound to their own cached verdict, and absent from Cache Storage URLs;
- an explicit service-worker update completed with active v3 and no waiting
  worker; the worker claimed the page and supported a functional offline
  reload.

Lighthouse 13.4.1 simulated mobile completed without runtime warnings:

| Category / metric | Result |
| --- | ---: |
| Performance | 100 |
| Accessibility | 100 |
| Best Practices | 100 |
| SEO | 100 |
| FCP | 1.1 s |
| LCP | 1.4 s |
| TBT | 50 ms |
| CLS | 0 |
| Speed Index | 1.1 s |
| Total transfer | 103 KiB |

## Non-blocking external state

The unadvertised factory checkout endpoint returns 404 because the paid product
is not enabled. The candidate accurately says sales are paused, presents no
checkout link or claim that it is currently purchasable, accepts no payment,
and keeps the complete core CLI free. Existing-token restore remains
functional. This factory-owned follow-up is not the reason for the verification
failure.

## Re-verification requirement

Authenticate every accepted JSON field, add a trusted-signer verification
path, cover both behaviors with release-installed CLI checks, then repeat clean
package/install, signature verification, and live deployment identity checks.
