# Repair handoff — ready for verification

## Scope and disposition

Repair work order `kube-permission-evidence-repair-2` addressed every finding
in `.factory/verification-2.md` for candidate
`9214065b8d33fdfc40de4aa9b21bccb4bd19c874`.

1. **Wildcard subresources:** the evaluator now mirrors Kubernetes RBAC's
   special `*/subresource` match. `*/scale` grants `deployments/scale` and
   `statefulsets/scale`, but not the parent resource or `*/status`.
2. **Unavailable paid offer:** the live-sale claim, `$49` price, dead checkout
   link, and unpublished-download promise were removed. The site states that
   sales are paused until the factory-owned Sociobot product and all promised
   assets are available. Existing-token restore remains supported. No billing
   or infrastructure was modified, in accordance with `AGENTS.md`.
3. **Verdict/token misbinding:** cached verdicts now include the exact token
   verified. Legacy unbound verdicts and verdicts for another token are
   ignored; a replacement return token is always verified for itself.
4. **License URL persistence:** service-worker cache v3 never stores a request
   containing the `license` query parameter. Activation deletes v2 and all
   other old caches, removing legacy query-bearing entries.

The researched brief, Rust CLI artifact, static deployment class, visual
system, complete free feature set, and previously passing behaviors are
unchanged.

## Exact regression coverage

- `src/evaluator.rs`: unit coverage for `*/scale` across two parent resources,
  plus negative wrong-subresource and parent-resource cases.
- `tests/access_matrix.rs`: bound-role integration coverage verifies decisions,
  summary counts, and causal grant preservation.
- `tests/cli.rs`: compiled `kpe report --json` reproduction of the verifier's
  rule and request; asserts 1 allowed / 1 denied and the emitted `*/scale`
  grant.
- `tests/site/landing.spec.ts`: browser coverage for token-bound verdict
  persistence; the exact `valid-one` → `invalid-two` replacement sequence;
  no query-bearing Cache Storage entry under a controlling worker; no live
  checkout claim; cache v3 update policy.

## Local verification — 2026-08-28 UTC

Commands and results:

```text
npm ci                                      PASS — 24 packages, 0 vulnerabilities
npm audit --audit-level=high                PASS — 0 vulnerabilities
npm test                                    PASS
  npm run typecheck                         PASS
  cargo fmt --all -- --check                PASS
  cargo clippy --all-targets -- -D warnings PASS
  Rust unit/integration tests               PASS — 14
  Chromium site tests                       PASS — 14
npm run build                               PASS — target/release/kpe + dist/site
cargo test --doc                            PASS — 1
cargo package --locked --allow-dirty        PASS — 25 files; staged crate compiled
cargo install --locked --path <staged>      PASS — clean consumer root
```

The clean installed package reported `kpe 0.1.0`, exposed the documented
non-interactive commands, and evaluated the packaged example as 2 allowed / 2
denied. The factory URL verifier passed locally in 535 ms with no console or
page errors, a title, `lang=en`, one h1, a main landmark, complete image alt
text, and labeled buttons.

Browser coverage includes desktop, 390×844 mobile, keyboard-only demo use,
44 px touch targets, serious/critical axe checks for all routes, offline
reload, immediate service-worker claiming, license return stripping, token
replacement, privacy-first-load behavior, and production response-policy
artifact assertions.

Lighthouse 13.0.1 simulated mobile against the production build:

| Category / metric | Result |
| --- | ---: |
| Performance | 100 |
| Accessibility | 100 |
| Best Practices | 100 |
| SEO | 100 |
| FCP | 0.9 s |
| LCP | 1.5 s |
| TBT | 0 ms |
| CLS | 0 |
| Total transfer | 103 KiB |

Production artifacts before deployment:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `target/release/kpe` | 1,591,264 | `a3ea03572828b46e4044797f4ad5a9e56db53c00d59f5b01e3888f2c0fad40f8` |
| `dist/site/index.html` | 10,678 | `24b615a12c1569623473540c247c1996d087feaadf849867b5bfe248d548f8f7` |
| main JS | 5,986 | `b99f639c4827b293e740d1357e12abccf58b14aad88d6fb2631f34d5be916e6a` |
| CSS | 14,183 | `8619e6173c6e700645b985320f2663551eef76a524e313ea5dfc94ce1de9dbfc` |
| service worker | 1,303 | `f7afc179252beee98b4dae126729b24fa45413d24bf677dce816d2dda6b84c94` |
| hero WebP | 93,322 | `b71aca102454a1d83c31d17969dceb8013f047edae8bc5e6a90990b23f0c12eb` |

JS, CSS, fonts (none), and hero image remain comfortably inside the supplied
budgets. No analytics, telemetry, CDN script, third-party font, cluster data,
or kubeconfig token is sent by the site.

## Deployment and live identity

Repair commit `aed9592545cdc55be1a49a92a7da928c14862136` was pushed to
`origin/main`. `/opt/fleet/lib/deploy-static.sh kube-permission-evidence
dist/site` completed successfully as Azure Static Web Apps deployment
`da86370f-1f0b-4727-95f8-7c078369c3fe`; the custom domain was `Ready` and
returned HTTPS 200.

Fresh downloads from <https://kube-permission-evidence.sociobot.in/> matched
the local SHA-256 values for HTML, JS, CSS, service worker, and hero image.
`/`, `/privacy/`, and `/terms/` return 200; HTTP redirects to HTTPS with 301.
Live responses include CSP, Permissions-Policy, HSTS, strict referrer policy,
and `nosniff`. HTML uses 30-second revalidation and `sw.js` is `no-cache,
no-store, must-revalidate`.

The factory live URL verifier passed in 634 ms with no console/page errors.
Fresh live Chromium at 1440×900 and 390×844 confirmed one h1/main, no page
overflow, no undersized controls, no checkout link, explicit paused-sale copy,
no serious/critical axe violations, and an offline reload. Cache Storage held
only `kpe-field-guide-v3` and contained no URL with a `license` parameter after
a controlled license-return navigation. A separate clean 390 px run confirmed
zero first-load local/session storage, no foreign-origin requests, skip-link
and keyboard demo operation, and reduced animation/transition durations of
0.01 ms.

Live Lighthouse 13.0.1 simulated mobile: Performance 97, Accessibility 100,
Best Practices 100, SEO 100; FCP 0.9 s, LCP 1.4 s, TBT 200 ms, CLS 0, and
103 KiB transferred.

## Known external follow-up

Field Kit sales intentionally remain paused. Factory operators may register
and enable the Sociobot product and publish the complete promised asset set in
a later release; only then should the checkout link and price be restored.
The unadvertised checkout endpoint still returns the independently reported
404, which is why no purchase action or paid-delivery claim is exposed.
Registry publishing is also factory-owned, so this worker prepared and
consumer-tested the crate but did not publish it.
