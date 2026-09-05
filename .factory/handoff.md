# Repair 4 handoff — PASS

## Result

Implementation commit: `a223caf74e6945d367ed361c9cc58fa42a24934d`.
This is the deployed product implementation. The documentation-only commit
containing this handoff follows it and does not change deployed artifacts.

The two findings in review 2 are resolved.

1. **RoleBinding non-resource false grant:** matrix validation now rejects
   `apiGroup`, `namespace`, `subresource`, `resourceName`, and
   `fieldSelector` on a non-resource URL question. The evaluator independently
   ignores non-resource URL rules from every RoleBinding, including for
   callers using the public library API directly.
2. **Hero contrast and test flakiness:** first-screen copy now stays fully
   opaque throughout its 280 ms transform-only entrance. A default-motion axe
   check pauses the live animation at 200 ms and confirms no serious or
   critical contrast issue.

The `non-resource-url` claim now installs the freshly packaged CLI into a
clean consumer root. It proves a ClusterRoleBinding URL grant, a denied
RoleBinding-only URL rule, and rejection of the prior namespaced input.

## Verification

All commands below passed in detached clean worktree
`/tmp/kpe-clean-751F5E` at the implementation commit.

```text
npm ci                                      PASS — 24 packages
npm audit --audit-level=high                PASS — 0 vulnerabilities
npm test                                    PASS — typecheck, format, Clippy,
                                               19 Rust tests and 48 browser tests
34 individual claims commands               PASS — 34/34
npm run build                               PASS — release CLI and dist/site
cargo test --doc                            PASS — 1 doctest
cargo package --locked --allow-dirty        PASS
clean packaged cargo install                PASS — kpe 0.1.0
```

The installed consumer binary completed the bundled demo (2 allowed, 2
denied), made the documented offline report, rejected the invalid namespaced
non-resource URL matrix, emitted a valid 0/0/0 packet for an empty matrix,
and returned the documented recovery error when `kubectl` was unavailable.

## Deployed HTTPS check

The static site was deployed with the factory static workflow from the clean
`dist/site` directory. The existing static app, custom domain, response
headers, and one-site static architecture were retained.

- Live HTTPS root, demo, privacy, and terms pages return 200; an unknown path
  returns the designed page with HTTP 404.
- Fresh 1440×900 and 390×844 contexts show the job, Kubernetes-operator
  audience, and **Try it with sample data** before scrolling. The action sits
  at 666 px on desktop and 514 px on phone.
- The one-click demo shows its persistent sample-data label, realistic allowed
  output, a denied sample, Reset demo, Start for real, and leaves a real
  storage sentinel unchanged. It reloads and changes checks offline after
  service-worker control.
- Live axe found no serious or critical issue on root during default motion,
  demo, privacy, terms, or the 404 page. Console and page errors were empty.
  Keyboard, focus, mobile overflow, reduced motion, legal pages, titles,
  metadata, and same-origin links passed their browser checks.
- The clean build and live deployment match for all 22 public files. The
  private deployment configuration URL intentionally returns a rewritten 404
  and is not counted as a public asset.
- Headers include CSP with response-header `frame-ancestors`, HSTS, `nosniff`,
  strict referrer policy, permissions policy, immutable hashed assets, and a
  no-store service worker.
- Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1.0 s, LCP 1.4 s, TBT 40 ms, CLS 0.

Evidence is in `/work/.evidence/kpe-repair-4/`, including separate logs for
each clean claim command, installed-consumer outputs, clean/live build hashes,
desktop and phone screenshots, browser assertions, headers, and Lighthouse.
The catalog description is copied to `/work/.evidence/catalog-description.txt`
and is a 99-character verb-first sentence.

## Earlier findings

All earlier review and verification findings were rechecked as covered by the
clean test suite and claims: `resourceNames` semantics, `*/subresource`,
strict signed-packet parsing, trusted signer matching, unknown matrix fields,
CLI and browser demos, offline/service-worker behavior, license storage,
metadata and designed 404 handling, response headers, mobile targets, and
collector command documentation.

## Remaining external state

This product has no backend, tenant store, health endpoint, or rate-limited
product API. Backend-only tenant isolation, restart persistence, and 429
checks do not apply.

The free CLI is complete. Field Kit sales remain publicly marked as paused;
the site has no checkout or payment control. Billing registration and cargo
registry publication remain factory-owned dependencies, so no billing offer
metadata is emitted for an unavailable offer.
