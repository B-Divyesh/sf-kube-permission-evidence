# Review 2 handoff — FAIL

## Result

Work order `kube-permission-evidence-review-2` reviewed implementation
`98eb121448edd957f8494df9577b7c47dcbc158b` and documentation head
`972d249d8519dd83e6a82a45709f2811ebf99184`.

**FAIL — 2 findings and 0 untested claims.** Product code was not changed.

1. High: a namespaced non-resource URL question can be falsely granted by a
   RoleBinding to a ClusterRole.
2. Medium: the default first-screen opacity animation temporarily lowers text
   contrast below WCAG AA and makes `npm test` timing-dependent.

See `.factory/review-2.md` for reproduction details and full evidence.

## What was reviewed

- Read the brief, visual thesis, demo contract, copy audit, 34-claim manifest,
  all earlier review and verification reports, README, and implementation.
- Opened the live site in fresh desktop and phone contexts before scrolling.
- Exercised the one-click populated sample, persistent label, allowed and
  denied output, reset, Start for real, and real-storage isolation.
- Checked keyboard operation, focus, mobile targets, 200% text, reduced
  motion, accessibility, privacy requests, offline reload, links, metadata,
  legal pages, and the designed HTTP 404.
- Compared all 22 public generated files with a fresh candidate build.
- Built and packaged a clean detached checkout and installed the crate into a
  new consumer root.
- Ran all 34 declared claim commands separately. All passed; F-01 shows the
  `non-resource-url` sandbox is incomplete.

## Verification summary

```text
npm ci                                      PASS
npm audit --audit-level=high                PASS — 0 vulnerabilities
npm test                                    FAIL — first-screen contrast timing
npm run build                               PASS
cargo test --doc                            PASS — 1
cargo package --locked --allow-dirty        PASS — 25 files
34 individual claim commands                PASS — 34/34
clean packaged cargo install                PASS — kpe 0.1.0
installed normal/invalid/recovery paths      PASS
installed namespaced non-resource case      FAIL — false allowed result
factory verify-url                           PASS
live generated artifact comparison           PASS — 22/22
live Lighthouse                              100/100/100/100
```

Live Lighthouse measured FCP 1.0 s, LCP 1.4 s, TBT 60 ms, and CLS 0. Built
payloads are 7.4 KiB JavaScript, 16.7 KiB CSS, and a 93.3 KiB hero image.

Evidence is stored under `/work/.evidence/review2/`.

## Required next steps

1. Reject `namespace` and other resource-only fields on non-resource URL
   questions, and prevent RoleBindings from granting non-resource URLs. Add a
   packaged-CLI regression to the `non-resource-url` claim.
2. Keep first-screen text fully opaque during entrance motion. Add a default-
   motion axe check while the animation is in progress.
3. Rerun `npm test`, every declared claim command, the installed package, and
   the live review after deployment.

Cargo registry publication and Field Kit sales remain paused factory tasks.
The product has no backend, so tenant isolation, restart persistence, health,
and 429 checks do not apply.
