# Verify Kubernetes RBAC access evidence — PASS

**Verdict: PASS**

- Work order: `kube-permission-evidence-review-3`
- Reviewed: 5 September 2026 UTC
- Findings: 0
- Untested public claims: 0
- Implementation reviewed: `a223caf74e6945d367ed361c9cc58fa42a24934d`
- Documentation baseline: `b6d684d820846dbd5e0ef8162f8be964aba5e357`
- Live URL: <https://kube-permission-evidence.sociobot.in/>

Commits after the implementation candidate change only `.factory/handoff.md`
and `.factory/verification-5.md`. All 22 public files from a fresh candidate
build match the live deployment byte for byte.

## Job, audience, and first action

The job is to trace a Kubernetes access question to every binding, role, and
rule that grants it, then write an audit packet. It is for Kubernetes operators
preparing audits or permission changes without changing cluster state. Before
scrolling, fresh 1440 × 900 desktop and 390 × 844 phone sessions showed the
job, audience, and **Try it with sample data**. The action started at 666 px on
desktop and 514 px on phone.

## Clean checkout and installed CLI

A detached checkout at the implementation commit was used. It remained clean.

```text
npm ci                                      PASS — 24 packages
npm audit --audit-level=high                PASS — 0 vulnerabilities
npm test                                    PASS — typecheck, formatting,
                                             Clippy, 19 Rust tests,
                                             48 browser tests
npm run build                               PASS — release CLI and dist/site
cargo test --doc                            PASS — 1 doctest
cargo package --locked --allow-dirty        PASS — 25 files, 41.0 KiB
clean packaged cargo install                PASS — kpe 0.1.0
```

The installed binary displayed useful help and ran `kpe demo` with an empty
`PATH`. It created a new directory containing the snapshot, matrix, Markdown,
and JSON packet. The packet contained four checks: two allowed and two denied.
It refused to reuse the directory.

Normal, invalid, boundary, and recovery checks passed. An empty matrix produced
a valid 0/0/0 packet. A mixed resource and non-resource question, an invalid
subject, a missing snapshot, and missing `kubectl` returned clear exit-1
errors. A matching trusted key verified a signed packet; a different trusted
key returned exit 1. The packaged non-resource claim also proved that a
ClusterRoleBinding grants `/healthz/*`, a RoleBinding does not, and a
namespaced non-resource question is rejected.

## Public claims

Every command in `.factory/claims.json` was run separately from the clean
checkout. All 34 passed.

| Claim group | Result |
| --- | --- |
| `point-in-time-proof` through `json-stdout` | 12 / 12 PASS |
| `exit-codes` through `server-version-recorded` | 12 / 12 PASS |
| `limitations-recorded` through `sales-paused` | 10 / 10 PASS |

The landing, demo, privacy, terms, README, and CLI help were cross-checked
against the manifest. No missing, duplicate, false, or untested public claim
remains. The checks cover the collector command boundary, token-free
snapshots, RBAC scope and matching behavior, every causal path, signing,
trusted signer checks, isolated browser state, first-party requests, offline
use, license-cache safety, and paused sales.

## Live desktop and phone checks

- The first action opened a realistic populated demo in one click. The page
  kept **Demo — sample data, nothing is saved** visible, showed the binding,
  role, and rule for an allowed check, and showed a denied check with no grant.
- Reset restored the first sample and cleared only `demo:kpe:` session state.
  Start for real cleared demo state. A seeded real-storage value stayed
  unchanged. All requests in both full demo flows stayed on the product origin.
- `/`, `/demo/`, `/privacy/`, and `/terms/` returned 200 with their own titles,
  `lang=en`, one h1, and one main landmark. `/missing-review-3` returned the
  designed **Page not found** page with HTTP 404. That deliberate 404 is
  expected and is not a defect.
- Axe found zero serious or critical issues on all five routes. No visible
  mobile target was smaller than 44 × 44 px. At 200% text size, the demo label,
  reset action, and selector remained available without horizontal overflow.
- Tab focused the skip link first. Its focus outline was a visible 3 px clay
  line. Enter, Space, and keyboard selection completed the sample path without
  a trap.
- With default motion paused at 200 ms, hero copy opacity remained 1 and Axe
  found no serious or critical issue. Reduced motion shortened the entrance to
  `0.01ms`.
- After service-worker control, a fresh phone session reloaded `/demo/`
  offline, showed the offline notice, and changed the selected sample.
- The factory URL check passed on all four 200 routes with no console or page
  errors. All 13 discovered HTTP links returned 200.
- Live responses include CSP with header-only `frame-ancestors`, HSTS,
  `nosniff`, strict referrer policy, and a permissions policy. The CSP permits
  only the documented Sociobot license endpoint beyond the product origin.
- Mobile Lighthouse scored 100 Performance, 100 Accessibility, 100 Best
  Practices, and 100 SEO. LCP was 1.5 s, TBT 70 ms, and CLS 0. Built JavaScript
  is 7.41 KB, CSS is 16.64 KB, the hero image is 93.32 KB, and no fonts are
  downloaded.

## Earlier findings

Every earlier review and verification finding, including low and medium
items, was inspected and rechecked.

| Earlier issue | Current evidence and disposition |
| --- | --- |
| `resourceNames` false grants for create and list/watch | Resolved by Rust regression tests, the individual claim, and packaged CLI checks. |
| Kubernetes `*/subresource` false denial | Resolved by Rust and individual claim tests. |
| RoleBinding false grant for non-resource URLs | Resolved by the installed-package claim; the false path is denied and namespaced URL input is rejected. |
| Unknown signed JSON fields were accepted | Resolved by strict-schema tests at report, check, grant, and signature levels. |
| No trusted signer requirement | Resolved by installed matching and mismatching trusted-key checks. |
| Misspelled matrix fields were accepted | Resolved by clean tests and packaged invalid-input checks. |
| Rust formatting and TypeScript checks failed | Resolved; both pass in `npm test`. |
| Cache, CSP, and permissions headers were missing | Resolved by live response inspection. |
| Mobile targets were below 44 px | Resolved by fresh phone scans on every route. |
| Service-worker updates were not immediate | Resolved in source with versioned cache, `skipWaiting`, and `clients.claim`; offline reload passed. |
| Paid checkout was unavailable | Resolved by honest paused-sales copy and no purchase action. |
| License verdict could be reused for another token | Resolved by the individual license claim and aggregate browser regression. |
| License query could enter Cache Storage | Resolved by the license claim and cache inspection. |
| CLI demo, browser sandbox, and demo documentation were missing | Resolved by the installed sample and fresh isolated browser flow. |
| Claims manifest and one-to-one tests were missing | Resolved; all 34 declared commands passed separately. |
| First-screen audience, sample action, and plain words were missing | Resolved by desktop and phone inspection and the copy audit. |
| Real 404, route metadata, legal skeleton, and footer were missing | Resolved on live routes and the deliberate 404 response. |
| Collector command documentation was incomplete | Resolved by the controlled collector claim and README list. |
| Hero entrance reduced text contrast | Resolved; text stays opaque during the default animation. |

## Scope

This is a static documentation site and a local CLI. It has no product
backend, tenant store, health endpoint, or live rate-limited API. Backend-only
tenant isolation, restart persistence, health, and 429/Retry-After checks do
not apply. Privacy requests route to the public issue tracker; there is no
product account or server-side user store. AI is not missing leverage because
the central access result must remain deterministic and inspectable.

There are no findings and no untested claims. **Verdict: PASS.**
