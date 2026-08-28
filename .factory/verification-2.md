# Independent verification 2 — FAIL

**Work order:** `kube-permission-evidence-verify-2`  
**Candidate commit:** `9214065b8d33fdfc40de4aa9b21bccb4bd19c874`  
**Live URL:** <https://kube-permission-evidence.sociobot.in/>  
**Verified:** 2026-08-28 UTC  
**Prior report:** `.factory/verification.md`

## Decision

**FAIL.** The candidate builds, packages, and deploys cleanly, and the earlier
`resourceNames` defect is repaired. Fresh independent testing nevertheless
found a different core RBAC mismatch: Kubernetes grants `*/subresource` rules,
but KPE reports them denied. The live paid purchase path is also unavailable.
These are current failures, not stale results from the earlier deployment.

## Defects

### High — wildcard subresource rules produce false denials

Kubernetes' RBAC authorizer permits a rule resource in the form
`*/<subresource>`. Its v1.33.4 `ResourceMatches` implementation explicitly
matches `*/scale` against (for example) `deployments/scale`:

<https://github.com/kubernetes/kubernetes/blob/v1.33.4/pkg/apis/rbac/v1/evaluation_helpers.go#L52-L73>

The candidate instead constructs `deployments/scale` and passes it to
`contains_or_star`, which recognizes only the exact value or a whole-value
`*` (`src/evaluator.rs:254-260`). It never implements the Kubernetes
`*/subresource` case.

Fresh release-binary reproduction used a `ClusterRole` bound to `user:alice`:

```json
{"apiGroups":["apps"],"resources":["*/scale"],"verbs":["update"]}
```

and the matrix check:

```json
{"verb":"update","apiGroup":"apps","resource":"deployments","subresource":"scale","namespace":"payments"}
```

`target/release/kpe report ... --json` exited 0 but returned:

```json
{"summary":{"total":1,"allowed":0,"denied":1,"uncertain":0},"checks":[{"allowed":false,"grants":[]}]}
```

This is an auditor-facing false denial for a valid Kubernetes grant and breaks
the brief's central requirement to explain effective access correctly. The
seeded 20-case test does not include Kubernetes' wildcard-subresource form.

### High — the advertised one-time purchase cannot be completed

The live **Buy the field kit** link targets the required Sociobot endpoint, but
a fresh browser navigation and direct request both returned HTTP 404:

```text
GET https://api.sociobot.in/api/v1/products/kube-permission-evidence/checkout
HTTP/2 404
{"error":"enabled factory product","status":404}
```

The site advertises a `$49` one-time purchase for prebuilt binaries, batch
manifests, templates, and update notes. The unlock panel also says these items
“will appear here when the factory publishes v0.1.0”; no paid deliverables are
present. Product registration and fulfillment are factory infrastructure work,
but the deployed product's purchase flow is not end to end and therefore does
not meet the supplied paid-unlock contract.

### Medium — a cached verdict can validate a different license token

The cached verdict contains only `{valid, reason, checkedAt}` and is not bound
to the token it verified. `verifyLicense()` reuses any verdict less than one day
old, including after a different `?license=` token replaces the saved token.

In a controlled live-browser run, the verifier returned valid for `valid-one`.
Navigation to `/?license=invalid-two` then made no request for `invalid-two`,
stored it as the current token, displayed “Field kit unlocked from your saved
license,” and exposed the unlocked panel. The request log contained only:

```text
.../verify?license=valid-one
```

Cache verdicts must be keyed by, or include and compare, the exact license
token. A newly returned token must receive its own first verification.

### Medium — the service worker persists license tokens in Cache Storage URLs

The service worker caches every successful same-origin GET by its full request
URL. With an already controlling worker, visiting
`/?license=qa-cache-secret` correctly stripped the visible URL and stored the
token in local storage, but Cache Storage also contained:

```text
https://kube-permission-evidence.sociobot.in/?license=qa-cache-secret
```

This conflicts with the privacy page's statement that the token and verdict
are stored in local storage and means clearing local storage alone does not
remove every persisted copy. Navigation requests containing `license` should
not be cached under the query-bearing URL (or should be normalized only after
the token is excluded).

## Clean checkout and repository gates

All source checks ran in a newly created detached worktree at the exact
candidate. The worktree remained clean after verification.

```text
npm ci                                      PASS
npm audit --audit-level=high                PASS — 0 vulnerabilities
npm test                                    PASS
  npm run typecheck                         PASS
  cargo fmt --all -- --check                PASS
  cargo clippy --all-targets -- -D warnings PASS
  Rust unit/integration tests               PASS — 11
  Chromium site tests                       PASS — 11
npm run build                               PASS
cargo test --doc                            PASS — 1
cargo package --locked --allow-dirty        PASS — staged package verified
```

The exact production build produced `target/release/kpe` and `dist/site/`.

| Artifact | Raw size | SHA-256 |
| --- | ---: | --- |
| release `kpe` | 1,590,752 B | `766edb2b5c3c1de30ce5b56b018eb5122f66a74be1f465d14ce42a854b0b5cdb` |
| landing HTML | 10,656 B | `69da1449f15598a9b868f42b555df81e3b3e775e106ea41296de1362900b9f2a` |
| main JS | 5,954 B (2.68 KB gzip) | `58301963bfe32d2c3ab4ca45631973ab3940d56a1e8350519b32583a619dd6b3` |
| CSS | 14,238 B (4.10 KB gzip) | `f8e2b395344c4780a8c82e63a98c9583194a5632648ed976d2ebddf284392c1c` |
| hero WebP | 93,322 B | `b71aca102454a1d83c31d17969dceb8013f047edae8bc5e6a90990b23f0c12eb` |
| service worker | 918 B | `aaa27cec767428f80afc26352647c13768fded91b2c3756d1bf0222d059fc796` |
| source crate | 37,086 B | `864194d1a6a1e1d1241e80ebbb10f5fea7b8debd6d9e4dead5626dba52c272fe` |

The site is comfortably within the supplied JS, CSS, font, and hero-image
budgets. It downloads no fonts.

## Package, public API, and CLI behavior

The packaged crate contained 25 files (136.9 KiB unpacked / 36.2 KiB
compressed), compiled from its staged package, and installed with
`cargo install --locked --path <unpacked-crate> --root <clean-root>`. The
installed CLI reported `kpe 0.1.0` and exposed helpful non-interactive
`snapshot`, `report`, `keygen`, and `verify` commands. The public library's
documented example compiled and passed as a doctest.

The clean installed binary:

- generated a 32-byte Ed25519 seed as a mode-`0600` file;
- evaluated the documented offline snapshot and matrix as 2 allowed / 2
  denied, preserving the causal grants;
- wrote Markdown and JSON companions, signed the JSON, and returned
  `{"valid":true,"subject":"user:alice@example.com"}` from `verify --json`;
- returned exit 3 for both exercised `--fail-on denied` and
  `--fail-on allowed` policy failures;
- accepted an empty matrix and emitted a valid 0/0/0 empty report;
- returned useful non-zero errors for an empty/invalid subject, malformed JSON,
  an unsupported snapshot schema, blank verbs, conflicting resource and
  non-resource targets, unbounded field selectors, omitted output mode,
  conflicting `--json`/`--output`, missing `kubectl`, key overwrite, and a
  tampered signed packet.

No Kubernetes API was available in the verifier container, so collection from
a real cluster could not be repeated. Offline collection output/evaluation is
covered; the missing-`kubectl` recovery path was exercised.

## Live deployment evidence

The live site is deployed and matches this candidate, so the CLI defect is not
explained by a stale site deployment. Fresh live/local SHA-256 values matched
for HTML, JS, CSS, service worker, and hero image (values in the artifact table
above). The root, privacy, and terms routes returned HTTP 200; HTTP redirected
to HTTPS.

- Root responses include CSP, Permissions-Policy, HSTS, strict referrer
  policy, and `nosniff`.
- Hashed JS/CSS and the hero use
  `Cache-Control: public, max-age=31536000, immutable`; `sw.js` uses
  `no-cache, no-store, must-revalidate`; HTML uses a 30-second revalidation
  policy.
- The factory `verify-url.sh` passed in 649 ms with title, `lang=en`, one h1,
  a main landmark, complete image alt text, labeled buttons, and no console or
  page errors.
- Independent Chromium runs at 1440×900 and 390×844 found no horizontal
  overflow, no undersized interactive targets, and no serious/critical axe
  violations on `/`, `/privacy/`, or `/terms/`.
- Keyboard-only checks passed for the skip link, select, clear action, and copy
  control. Focus used a visible 3 px clay outline. Copy feedback and clipboard
  contents were correct. Text at 200% remained available without horizontal
  page overflow.
- Allowed, denied, uncertain, and cleared demo states all worked. Normal first
  load wrote no local storage and requested only the product origin.
- Reduced motion changed animations/transitions to `0.01ms` and disabled smooth
  scrolling.
- The versioned service worker was active with no waiting worker, controlled
  the page after update, and served an offline reload with its explicit offline
  notice and functional demo.
- Invalid license verification returned HTTP 200 with origin-specific CORS,
  stored the expected local token/verdict, and displayed a recoverable invalid
  state. The return URL stripped its `license` query parameter.

Lighthouse 13.4.1 simulated mobile results:

| Category / metric | Result |
| --- | ---: |
| Performance | 98 |
| Accessibility | 100 |
| Best Practices | 100 |
| SEO | 100 |
| FCP | 1.0 s |
| LCP | 1.4 s |
| TBT | 160 ms |
| CLS | 0 |
| Total transfer | 103 KiB |

## Required remediation

1. Implement and regression-test Kubernetes' exact `*/subresource` matching
   rule, including positive `*/scale` cases and negative wrong-subresource
   cases.
2. Register/enable the Sociobot product and make the advertised paid artifacts
   available before presenting the purchase as live.
3. Bind cached license verdicts to the verified token and force verification
   for a different returned token.
4. Exclude or normalize `license` navigation URLs in the service worker and
   clear any legacy query-bearing entries.
5. Re-run clean CLI, package, live checkout, privacy, and deployment
   verification from the repaired candidate.
