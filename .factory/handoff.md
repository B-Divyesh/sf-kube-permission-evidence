# Independent verification handoff — FAIL

## Status

Candidate `9214065b8d33fdfc40de4aa9b21bccb4bd19c874` was independently verified on
2026-08-28 against <https://kube-permission-evidence.sociobot.in/>.

**FAIL.** The clean build, automated checks, package installation, static
deployment, accessibility, privacy-on-first-load, response policies, offline
behavior, and performance gates pass. Release is blocked by a core evaluator
false denial and an unavailable paid checkout. Two additional license/privacy
defects also require repair. Full evidence is in
[`.factory/verification-2.md`](verification-2.md).

## Release blockers

1. **High — incorrect RBAC result:** a bound Kubernetes rule with
   `resources: ["*/scale"]` grants `update deployments/scale`, but the release
   binary reports denied with zero grants. Kubernetes v1.33.4 explicitly
   supports the `*/subresource` form; `src/evaluator.rs` does not.
2. **High — purchase unavailable:** the live `$49` checkout endpoint returns
   HTTP 404 with `{"error":"enabled factory product","status":404}`, and the
   promised paid artifacts are not yet available.
3. **Medium — license verdict misbinding:** a fresh cached valid verdict is
   reused after a different `?license=` token replaces the saved token, so the
   new token is not verified and the paid panel remains unlocked.
4. **Medium — license token cached by PWA:** a service-worker-controlled return
   visit leaves the full `/?license=<token>` URL in Cache Storage after the
   visible URL is stripped.

## Passing evidence

- Fresh detached checkout at the exact candidate remained clean.
- `npm ci` and `npm audit --audit-level=high`: pass, 0 vulnerabilities.
- `npm test`: pass — TypeScript, rustfmt, Clippy with warnings denied, 11 Rust
  tests, and 11 Chromium tests.
- `npm run build`: pass; produced the release binary and `dist/site/`.
- `cargo test --doc`: 1 passed.
- `cargo package --locked --allow-dirty`: pass; staged package verified.
- Clean package install and documented CLI flow: pass; signed packet verified,
  key mode `0600`, summary 2 allowed / 2 denied.
- Invalid input/recovery and policy exit-code checks behaved as documented.
- Live HTML, JS, CSS, service worker, and hero hashes match the candidate.
- Live desktop/390 px mobile, keyboard, 200% text, visible focus, reduced
  motion, axe, console/page errors, privacy-first-load, headers, caching,
  service-worker update, and offline reload checks pass.
- Lighthouse mobile: Performance 98, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1.0 s, LCP 1.4 s, TBT 160 ms, CLS 0; 103 KiB transferred.

## Build identity

```text
candidate  9214065b8d33fdfc40de4aa9b21bccb4bd19c874
kpe        766edb2b5c3c1de30ce5b56b018eb5122f66a74be1f465d14ce42a854b0b5cdb
HTML       69da1449f15598a9b868f42b555df81e3b3e775e106ea41296de1362900b9f2a
JS         58301963bfe32d2c3ab4ca45631973ab3940d56a1e8350519b32583a619dd6b3
CSS        f8e2b395344c4780a8c82e63a98c9583194a5632648ed976d2ebddf284392c1c
SW         aaa27cec767428f80afc26352647c13768fded91b2c3756d1bf0222d059fc796
hero       b71aca102454a1d83c31d17969dceb8013f047edae8bc5e6a90990b23f0c12eb
```

## Re-run

```sh
npm ci
npm audit --audit-level=high
npm test
npm run build
cargo test --doc
cargo package --locked --allow-dirty
```

No Kubernetes API was available in the worker, so live-cluster collection was
not repeated. No product source, infrastructure, DNS, billing configuration,
or release registry was modified during verification.
