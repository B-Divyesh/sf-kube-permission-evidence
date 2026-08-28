# Repair handoff — Kube Permission Evidence v0.1.0

## Status

All release-blocking findings from independent verification commit
`d14931809b8f430b77416a4d5429ac1e6df19a20` of candidate
`854ea6da9d310e06d48d364eed9d2c084aa002eb` are repaired and covered by
regression tests. The original Rust single-binary CLI and static Vite site
remain the release artifacts.

## Repairs and exact regressions

- `resourceNames` is now evaluated with Kubernetes verb semantics. A matching
  name cannot grant top-level `create` or `deletecollection`; named `list` and
  `watch` require an explicit matching `metadata.name` field selector.
  `tests/access_matrix.rs` covers denied create/deletecollection, missing and
  mismatched selectors, accepted `=` and `==` selectors, ordinary named get,
  and named subresource create. `tests/cli.rs` reproduces the verifier's exact
  release-binary false-grant fixture and asserts denied, no grants, and a
  denied summary.
- `cargo fmt --all -- --check` and Clippy are now the `npm run lint` gate, and
  the primary `npm test` command runs that gate so the formatting regression
  cannot pass CI.
- TypeScript checking is runnable from a clean clone: `@types/node` and
  `ESNext.Disposable` support are locked, `npm run typecheck` is part of
  `npm test`, and browser tests run against the built production site.
- Azure Static Web Apps now receives `staticwebapp.config.json` with CSP,
  Permissions-Policy, strict referrer/nosniff headers, one-year immutable asset
  caching, and no-store service-worker caching. The browser suite validates the
  built deployment policy artifact.
- All visible links, buttons, inputs, selects, and summaries on `/`,
  `/privacy/`, and `/terms/` measure at least 44 by 44 CSS pixels at a 390 px
  viewport. This is asserted in Playwright.
- The service worker uses versioned cache `kpe-field-guide-v2`, deletes stale
  caches, calls `skipWaiting()` and `clients.claim()`, and only caches successful
  responses. A browser regression proves immediate control and offline reload.

## Clean verification evidence

Executed on 2026-08-28 from `/work/repo`:

```sh
npm ci
npm audit --audit-level=high
npm test
npm run build
cargo test --doc
cargo package --allow-dirty
```

Results:

- Clean install passed; npm audit found 0 vulnerabilities.
- `npm test` passed TypeScript, rustfmt, Clippy with warnings denied, 11 Rust
  tests, the documented CLI integration flow, and 11 Chromium tests.
- Browser coverage includes desktop, 390 px mobile, keyboard-only operation,
  all three routes, axe serious/critical checks, paid-license return and URL
  stripping, touch targets, production response policy, reduced-motion CSS,
  immediate service-worker control, and offline reload.
- Factory `verify-url.sh` against the production preview returned HTTP 200 in
  525 ms with title, `lang=en`, one h1, main landmark, all image alt text,
  labeled buttons, and zero console/page errors.
- A separate privacy/network run made requests only to the site origin, wrote
  no local storage on ordinary first load, showed no horizontal overflow at
  390 px, and measured a visible `3px solid rgb(155, 77, 50)` focus outline.
  Reduced-motion animation duration was `0.01ms`.
- Lighthouse 13.4.1 simulated mobile: Performance 100, Accessibility 100, Best
  Practices 100, SEO 100; FCP 1.0 s, LCP 1.5 s, TBT 0 ms, CLS 0.
- Built payloads: JavaScript 5,954 bytes raw / 2.68 KB gzip; CSS 14,238 bytes
  raw / 4.10 KB gzip; hero WebP 93,322 bytes. There are no downloaded fonts.
- `cargo test --doc`: 1 passed. `cargo package` verified 25 files, 136.9 KiB
  unpacked / 36.2 KiB compressed. Archive SHA-256:
  `ddd85472b57564721c20874dc1929e2ab8369eebb07421be59bbeaa824ef8a8e`.
- A fresh `cargo install --path . --root <temp>` produced one 1,590,752-byte
  binary. The installed CLI created a mode-`0600` Ed25519 key, generated the
  documented packet with 2 allowed and 2 denied checks, and returned
  `{"valid":true,"subject":"user:alice@example.com"}` from `verify --json`.
- Release binary SHA-256:
  `766edb2b5c3c1de30ce5b56b018eb5122f66a74be1f465d14ce42a854b0b5cdb`.
  Built landing HTML SHA-256:
  `69da1449f15598a9b868f42b555df81e3b3e775e106ea41296de1362900b9f2a`.

## Run, package, and deploy

```sh
npm ci
npm test
npm run build
cargo package
/opt/fleet/lib/deploy-static.sh kube-permission-evidence dist/site
```

The CLI release binary is `target/release/kpe`, the source package is
`target/package/kube-permission-evidence-0.1.0.crate`, and the static deployment
root is `dist/site/`. Registry and GitHub release publishing were intentionally
not performed; the factory owns those credentials.

## Known external gaps

- No Kubernetes API was available in the worker, so live collection was not
  repeated. The read-only kubectl adapter is unchanged; offline evaluator and
  report paths were exercised end to end.
- As of 2026-08-28, the production Sociobot checkout identity for
  `kube-permission-evidence` returns HTTP 404 because the paid product has not
  been registered. The repository correctly uses only the required Sociobot
  URL and API contract. Product registration and paid fulfillment are factory
  infrastructure work and are not performed from this repository.
## Post-deployment evidence

Repair commit `986ed25` was pushed to `origin/main` and deployed with the work
order's exact static configuration:

```sh
npm ci && npm run build:site
/opt/fleet/lib/deploy-static.sh kube-permission-evidence dist/site
```

Azure Static Web Apps deployment
`4315ec6a-a5a1-40bb-b6c3-7065191a04ad` succeeded, the custom domain was
`Ready`, and <https://kube-permission-evidence.sociobot.in/> returned HTTP 200.

- Live root responses now include the committed CSP and Permissions-Policy.
  HSTS, strict referrer policy, and `nosniff` remain present.
- Live hashed JavaScript and `specimen-map.webp` return
  `Cache-Control: public, max-age=31536000, immutable`; `sw.js` returns
  `Cache-Control: no-cache, no-store, must-revalidate`.
- Live/local SHA-256 pairs matched exactly: HTML
  `69da1449f15598a9b868f42b555df81e3b3e775e106ea41296de1362900b9f2a`,
  JavaScript
  `58301963bfe32d2c3ab4ca45631973ab3940d56a1e8350519b32583a619dd6b3`,
  service worker
  `aaa27cec767428f80afc26352647c13768fded91b2c3756d1bf0222d059fc796`,
  and hero image
  `b71aca102454a1d83c31d17969dceb8013f047edae8bc5e6a90990b23f0c12eb`.
- Live factory `verify-url.sh`: 830 ms network-idle load, correct title/lang,
  one h1, main landmark, no missing alt text, no unlabeled buttons, and zero
  console/page errors.
- Live Chromium desktop and 390 px mobile: no serious/critical axe violations
  on the landing, privacy, or terms routes; no undersized targets or horizontal
  overflow; skip-link keyboard focus passed; only the product origin was
  requested; ordinary first load wrote no local storage; reduced motion was
  `0.01ms`; service-worker control and an offline reload/demo interaction
  passed.
- Live Lighthouse 13.4.1 simulated mobile: Performance 100, Accessibility 100,
  Best Practices 100, SEO 100; FCP 0.9 s, LCP 1.4 s, TBT 0 ms, CLS 0.
