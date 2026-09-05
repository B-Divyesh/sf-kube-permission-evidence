# Review Kubernetes RBAC evidence packets — FAIL

## Verdict

**FAIL.** This review found **2 findings**: 1 high and 1 medium. All 34
declared claim commands were run individually, so there are **0 untested
claims**. One declared claim is nevertheless false on an untested edge of its
sandbox.

- Work order: `kube-permission-evidence-review-2`
- Reviewed: 5 September 2026 UTC
- Live URL: <https://kube-permission-evidence.sociobot.in/>
- Implementation candidate: `98eb121448edd957f8494df9577b7c47dcbc158b`
- Documentation head before this review:
  `972d249d8519dd83e6a82a45709f2811ebf99184`
- Reason for different SHAs: commits after `98eb121` change only factory
  reports. All 22 public generated site files match a fresh implementation
  build byte-for-byte.

## Job, audience, and first action before scrolling

- Job shown: trace Kubernetes access to every granting rule.
- Audience shown: Kubernetes operators preparing audits or permission
  changes without changing cluster state.
- First action on desktop and phone: **Try it with sample data**.

The action begins at 666 px in a 900 px desktop viewport and 514 px in an
844 px phone viewport. It is visible before scrolling in both fresh contexts.

## Findings

### F-01 — High — a RoleBinding can produce a false non-resource URL grant

The clean, packaged `kpe 0.1.0` accepted this matrix question:

```json
{"verb":"get","nonResourceURL":"/healthz/ready","namespace":"payments"}
```

The snapshot contained a `RoleBinding` in `payments` that referenced a
`ClusterRole` with `get` on `/healthz/*`. KPE returned exit 0, `allowed: true`,
and a causal grant through that RoleBinding.

That access is not granted by Kubernetes. A RoleBinding grants permissions
only within its namespace. Non-resource endpoints such as `/healthz` are not
namespaced and require cluster-wide binding. Kubernetes states both rules in
its RBAC reference:
<https://kubernetes.io/docs/reference/access-authn-authz/rbac/>.

The evaluator first treats the supplied `namespace` as a match for the
RoleBinding, then evaluates the non-resource URL rule. Matrix validation does
not reject `namespace` or other resource-only fields on a non-resource
question.

This also makes the public `non-resource-url` claim false for an accepted
input. Its declared test passes because it checks only a ClusterRoleBinding;
it does not test a RoleBinding or reject a namespaced non-resource question.

Required repair: reject resource-only fields on non-resource questions and
never evaluate non-resource URLs through RoleBindings. Add an installed-binary
regression for this exact false-positive path.

### F-02 — Medium — the first-screen fade drops text below required contrast

The default first-screen animation applies opacity to the complete
`.hero-copy` block for 500 ms. A deterministic axe probe against the live page
paused that real animation at several points:

| Animation time | Block opacity | Result |
| ---: | ---: | --- |
| 100 ms | 0.308 | Serious contrast violation; body text 1.9:1 |
| 200 ms | 0.571 | Serious contrast violation; body text 3.8:1 |
| 300 ms | 0.785 | Serious contrast violation; small text 3.4:1 |
| 400 ms | 0.938 | No violation |
| 500 ms | 1 | No violation |

This is visible default-motion content, not a disabled or hidden state. It
fails the required 4.5:1 text contrast during the first 300 ms.

The clean `npm test` run caught the same timing defect and exited 1: axe
measured `.action-note` and the three fact labels at 4.46:1 while the animation
was finishing. The run ended with 46 passing browser tests and 1 failure.
Later repeats passed after landing at other animation times, which confirms a
timing-dependent gate rather than removing the defect.

Reduced motion is correctly shortened to 0.01 ms. Required repair: do not
fade readable text; animate transform only or keep text fully opaque. Test axe
at an in-progress default-motion state.

## Declared claims

Every command in `.factory/claims.json` was run separately from the clean
checkout. All 34 commands exited 0, and every ID appears exactly once in the
claim test source.

| Claim IDs | Command result |
| --- | --- |
| `point-in-time-proof` through `json-stdout` | 12 PASS |
| `exit-codes` through `server-version-recorded` | 12 PASS |
| `limitations-recorded` through `sales-paused` | 10 PASS |

Untested claim count is zero. F-01 remains a finding because the
`non-resource-url` test is incomplete and its public claim is false on an
accepted matrix shape.

## Clean checkout, build, package, and installed CLI

The detached checkout at `98eb121` stayed free of source edits.

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 24 packages |
| `npm audit --audit-level=high` | PASS — 0 vulnerabilities |
| `npm test` | **FAIL** — 46 browser tests pass; F-02 fails axe |
| `npm run build` | PASS — release binary and `dist/site/` |
| `cargo test --doc` | PASS — 1 doctest |
| `cargo package --locked --allow-dirty` | PASS — 25 files, 40.3 KiB compressed |
| Clean packaged `cargo install --locked --path ...` | PASS — `kpe 0.1.0` |

The installed artifact passed `--help`, `--version`, the bundled demo,
existing-directory refusal, offline JSON and file reports, exit 0/1/3 paths,
malformed JSON, invalid subjects, unknown matrix fields, an empty matrix,
missing kubectl, owner-only key creation, overwrite refusal, trusted-signer
match and mismatch, unknown signed fields, and changed signed content.

The installed artifact then reproduced F-01. Normal, invalid, boundary, and
recovery paths do not cancel a definite false access grant.

## Live desktop, phone, accessibility, privacy, and offline checks

Fresh 1440×900 and 390×844 Chromium contexts exercised the live site.

- One click opens `/demo/` with an allowed sample and the
  **Demo — sample data, nothing is saved** label.
- The output names Alice, `payments / alice-secrets`, the RoleBinding, Role,
  rule, and effective access. Another sample shows a denied result.
- Reset restores the first allowed check and clears `demo:kpe:` state. Start
  for real clears demo state and returns home. A real-storage sentinel remains
  unchanged throughout.
- Tab reaches the skip link first. Its focus outline is a visible 3 px clay
  line. Selection, clear, reset, and exit work from the keyboard.
- Phone pages have no horizontal overflow and no visible target under 44 px.
  Text resized to 200% keeps the demo label and reset control available.
- Reduced motion changes entrance and control durations to 0.01 ms and uses
  automatic scrolling.
- After service-worker control, the phone context reloads `/demo/` offline,
  shows the offline notice, and changes the selected sample successfully.
- Root, demo, privacy, and terms return 200 with correct route titles,
  `lang=en`, one h1, one main landmark, canonical metadata, and image alt text.
- `/missing-review-2` returns the designed **Page not found** page with HTTP
  404. Its single browser 404 console entry is expected, not a defect.
- All internal links, the example download, the source repository, and the
  privacy support link resolve.
- A clean normal load contacts only the product origin. An existing license
  can contact only the documented Sociobot verification origin. There are no
  analytics, trackers, CDN fonts, or third-party scripts.
- Axe after animation completion reports no serious or critical issue on the
  five routes. F-02 records the separate in-progress animation probe.
- The factory URL verifier passes. Security headers include CSP, HSTS,
  `nosniff`, referrer policy, and permissions policy.

Live mobile Lighthouse scores 100 for Performance, Accessibility, Best
Practices, and SEO. FCP is 1.0 s, LCP 1.4 s, TBT 60 ms, and CLS 0. The build
ships 7.4 KiB JavaScript, 16.7 KiB CSS, and a 93.3 KiB hero image.

This is a static site and local CLI. It has no product backend, tenant store,
health endpoint, or rate-limited live API, so backend isolation, persistence,
and 429 checks do not apply.

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| `resourceNames` false grants | Resolved by tests and installed-artifact checks. |
| Rust formatting and TypeScript gate | Resolved; both pass before the browser phase. |
| Cache and security headers | Resolved in live responses. |
| Mobile targets below 44 px | Resolved in the fresh phone scan. |
| Service-worker update and offline behavior | Resolved in a controlled offline reload. |
| `*/subresource` false denial | Resolved by claim and installed behavior tests. |
| Unavailable paid checkout | Resolved by honest paused-sales copy and no payment action. |
| License verdict reused for another token | Resolved by the claim test. |
| License token stored in Cache Storage URLs | Resolved by the claim test. |
| Added signed JSON fields accepted | Resolved by strict verification and installed tamper tests. |
| No trusted-signer requirement | Resolved by matching and mismatching trusted-key tests. |
| Misspelled matrix fields accepted | Resolved by installed rejection. |
| Missing CLI and browser demos | Resolved by populated, isolated demo paths. |
| Missing claims manifest | Resolved; all 34 commands ran. F-01 is a newly found incomplete claim case. |
| Unclear first-screen and metaphor copy | Resolved by current copy and copy audit. |
| Missing real 404 | Resolved; an unknown route returns the designed HTTP 404. |
| Missing route metadata and shared skeleton | Resolved on every checked route. |
| Incomplete collector command list | Resolved by documentation and the controlled collector claim. |

The fresh review therefore does not uphold verification 4's PASS. Its prior
findings remain closed, but F-01 and F-02 are new current findings.

## AI feature check

No AI feature is missing. The core job needs deterministic, inspectable RBAC
evaluation. A model-generated access conclusion would weaken the evidence.

## Evidence

Evidence is under `/work/.evidence/review2/`: individual claim logs, installed
CLI output, the false-grant packet, live desktop and phone screenshots, route
and offline assertions, animation contrast samples, artifact hashes, the URL
verifier output, and Lighthouse JSON.

Required next work is limited to F-01 and F-02, followed by a clean rerun of
all gates and claims and a fresh installed-artifact and live check.
