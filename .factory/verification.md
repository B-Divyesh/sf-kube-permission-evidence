# Independent verification — FAIL

**Work order:** `kube-permission-evidence-verify-1`  
**Candidate source commit:** `854ea6da9d310e06d48d364eed9d2c084aa002eb`  
**Live URL checked:** <https://kube-permission-evidence.sociobot.in/>  
**Verification date:** 2026-08-28

## Decision

**FAIL.** The CLI can issue a definite, auditor-facing false positive for
Kubernetes RBAC. This breaks the brief's core job: correctly explain effective
access without making a misleading access claim. The live site is deployed and
matches this candidate, so this is not a deployment-only failure.

## Blocking defect

### High — `resourceNames` produces false grants for operations Kubernetes cannot restrict by name

`src/evaluator.rs` treats a supplied `resourceName` as sufficient for every
verb. A fresh release-binary test used a `ClusterRole` rule granting
`create pods` with `resourceNames: [approved]`, bound to `user:alice`. KPE
reported `allowed: true`, `denied: 0`, and a causal grant for this request:

```json
{"verb":"create","resource":"pods","namespace":"payments","resourceName":"approved"}
```

Kubernetes' RBAC reference says: “You cannot restrict `deletecollection` or
top-level `create` requests by resource name”; a name may not be known at
authorization time. See <https://kubernetes.io/docs/reference/access-authn-authz/rbac/#referring-to-resources>.
KPE must return denied/unsupported/uncertain for this case, not a definite
grant. The same evaluator also reports an allowed `list` constrained by
`resourceName` even though its matrix has no field-selector representation;
Kubernetes requires a matching `metadata.name` field selector for named
`list`/`watch` authorization. This is security-significant because the report
is designed to be audit evidence.

## Other defects

### Medium — committed code fails the Rust formatting quality gate

`cargo fmt --all -- --check` exits 1 and prints formatting diffs for
`tests/cli.rs`. No source was changed during verification.

### Medium — TypeScript project check is not runnable cleanly

`npx tsc --noEmit` exits non-zero. The committed `tsconfig.json` includes
`vite.config.ts`, but `@types/node` is absent; it also lacks the required
library support for Rollup's `Symbol.asyncDispose`. Vite transpilation succeeds
but does not replace a type check.

### Medium — production cache/policy headers do not meet the static-product contract

The repository's `site/public/_headers` requests immutable caching for
`/assets/*` and `/specimen-map.webp`, but the live response for every checked
asset is `Cache-Control: public, must-revalidate, max-age=30`. The live
responses also have no `Content-Security-Policy` or `Permissions-Policy`.
HSTS, `X-Content-Type-Options: nosniff`, and a strict referrer policy are
present. The deployed static host configuration must honor the intended
long-lived hashed-asset caching and add an appropriate CSP.

### Medium — several mobile interactive targets are under the required 44 px height

At a 390 px viewport, the visible brand link measures 147×40 px and the Copy
button 64×36 px; footer/legal links are also smaller. This misses the supplied
touch-target baseline, although keyboard operation and focus indication work.

### Low — service worker updates are not immediate

`site/public/sw.js` has neither `skipWaiting()` nor `clients.claim()` and uses
the fixed cache name `kpe-field-guide-v1`. Offline reload works, but a newly
deployed worker can remain waiting while an existing client is open; versioned
cache/update handling should be made explicit before relying on it for release
updates.

## What passed

### Clean install, tests, package, and build

- `npm ci` completed; `npm audit --audit-level=high` reported 0 vulnerabilities.
- `npm test` passed: 8 Rust tests and 6 Playwright tests.
- Exact production command `npm run build` passed and produced
  `target/release/kpe` plus `dist/site/`.
- `cargo clippy --all-targets -- -D warnings` passed.
- `cargo test --doc` passed.
- `cargo package --allow-dirty` passed, verified the package, and produced
  `target/package/kube-permission-evidence-0.1.0.crate` (34.3 KiB compressed).
- A clean consumer `cargo install --path . --root <temp>` succeeded. Its
  installed `kpe` generated a mode-`0600` Ed25519 key, created a signed offline
  packet from the documented examples (2 allowed, 2 denied), and verified it
  with `kpe verify --json`.

### CLI behavior exercised

- Normal offline report, Markdown/JSON output, signing, and verification: pass.
- `--fail-on denied` exits 3 as documented: pass.
- Invalid subject, malformed matrix, malformed ServiceAccount subject, absent
  `kubectl`, and invalid verify input return exit 1 with useful recovery text:
  pass.
- The seeded 20-case evaluator test passes, but it does not cover the
  Kubernetes `resourceNames` verb semantics above.

### Live deployment, privacy, accessibility, and performance

- Live `/` HTML SHA-256 equals the freshly built `dist/site/index.html`:
  `d9d835a8f6d8698a20d5c1294c660066c72b0ad57a204551a70991b56ce0c594`.
  The hashed main JavaScript also matched byte-for-byte:
  `b469a1884a81f768caae464bda3c789d61a5388b503cc7a4fe380579de8e776a`.
- Chromium checks at desktop and 390 px found no horizontal page overflow;
  `<main>` and exactly one `<h1>` are present. The mobile 390 px normal flow,
  denied-demo path, keyboard-selected control, and visible 3 px focus ring
  passed. Reduced motion reduces the hero animation to `0.01ms`.
- Live axe had no serious or critical violations; there were no console or page
  errors. The initial normal-page request set contained only the product origin
  (no analytics, CDN fonts, trackers, or other outbound calls).
- The service worker controlled a subsequent page and an offline reload kept
  the title and explicit offline banner available.
- Live Lighthouse mobile (13.4.1): Performance 97, Accessibility 100, Best
  Practices 100, SEO 100; FCP 2.1 s, LCP 2.1 s, TBT 90 ms, CLS 0.
- Built payloads are within budget: JS 5,954 bytes raw / 2,694 gzip; CSS
  14,093 bytes raw / 4,080 gzip; hero WebP 93,322 bytes. No downloaded fonts.

## Required remediation and re-verification

1. Model Kubernetes `resourceNames` semantics by verb: reject or mark
   top-level `create` and `deletecollection` name-constrained checks as not
   authorized; add an explicit field-selector input/state before allowing
   named `list`/`watch`. Add regression tests against the documented rules.
2. Format `tests/cli.rs`; make a clean TypeScript check runnable and add it to
   the repository scripts/CI.
3. Correct deployed cache and security response policies, and enlarge all
   mobile interactive targets to 44 px or more.
4. Version/update the service-worker cache deliberately, then rerun this
   verification from the remediated commit.
