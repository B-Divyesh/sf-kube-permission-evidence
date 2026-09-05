# Review 1 handoff — FAIL

## Result

Review work order `kube-permission-evidence-review-1` is **FAIL** for live
implementation `aed9592164633ab4ef8f7ab9e5da082368f4ac7e` at
<https://kube-permission-evidence.sociobot.in/>. The documentation head entering
the review was `3c058becd13b80c2d105ed8b89df73414af521ea`; later commits after the
implementation contain reports only.

The full evidence is in `.factory/review-1.md`. The result is 10 findings
(5 high, 5 medium) and 34 public claim groups without declared claim tests.

## Main open work

- Reject unauthenticated unknown fields in signed packets.
- Let auditors require an expected signer key or fingerprint.
- Reject unknown access-matrix fields instead of silently changing a question.
- Add the required CLI sample demo, `/demo` state, persistent sample label,
  reset, exit action, and `.factory/demo.md`.
- Add `.factory/claims.json` with one `@claim:` test per public promise.
- Correct the incomplete collector command list.
- Replace metaphor copy and add `.factory/copy-audit.md`.
- Add a real 404 response/page, full metadata, consistent legal-page footer and
  navigation, Param Factory credit, and build identifier.

## What was verified

From a clean detached checkout of the implementation candidate:

```text
npm ci                                      PASS
npm audit --audit-level=high                PASS — 0 vulnerabilities
npm test                                    PASS — 14 Rust + 14 Chromium tests
npm run build                               PASS — release CLI + dist/site
cargo test --doc                            PASS — 1
cargo package --locked --allow-dirty        PASS — 25 files
clean packaged cargo install                PASS — kpe 0.1.0
offline report/sign/verify                  PASS for unchanged packet
unknown signed fields                       FAIL — accepted at four levels
trusted signer enforcement                  FAIL — unavailable
unknown matrix field                        FAIL — changed a verdict silently
CLI demo                                    FAIL — unavailable
```

The installed artifact also passed documented output, exit-code, invalid-input,
missing-tool, unwritable-output, key-overwrite, and unsigned-packet recovery
paths. A controlled kubectl process proved that collection stayed read-only.
It also showed the two safe calls missing from README's exclusive list.

Fresh live desktop and 390 px phone contexts covered the root, privacy, terms,
demo fallback, missing route, specimen states, keyboard, focus, reduced motion,
license failure recovery, storage, requests, links, service-worker update, and
offline reload. Playwright axe found no serious or critical issue. There were
no console errors, undersized targets, horizontal overflow, or third-party
first-load requests.

Lighthouse mobile scored 100 for Performance, Accessibility, Best Practices,
and SEO. FCP was 1.01 s, LCP 1.37 s, TBT 41 ms, CLS 0, and transfer about
103 KiB. JavaScript, CSS, hero image, and font budgets pass.

## Files changed by this review

- `.factory/review-1.md`
- `.factory/handoff.md`
- `/work/.evidence/qa-report.md` and `qa-result.json`
- `/work/.evidence/live/`, `verify-url/`, and `kubectl-harness/` supporting
  evidence

No product source, site source, tests, deployment, infrastructure, secrets, or
other service was changed.

## How to repeat

```sh
npm ci
npm test
npm run build
cargo test --doc
cargo package --locked --allow-dirty
```

Install the staged crate into an empty Cargo root, then run `kpe --help`, the
README offline example, both `--fail-on` modes, signing, tamper checks, and
two-key verification. Review `/demo` and a missing route in new desktop and
phone browser contexts. Run the factory URL verifier, Playwright axe, and
Lighthouse against the live URL.

## Deployment and external state

The current live artifacts match the implementation candidate. Field Kit sales
remain honestly paused and no checkout is shown. No deployment is requested by
this report-only work order.
