# Repair 3 handoff — PASS

## Result

Work order `kube-permission-evidence-repair-3` resolves every finding in
`.factory/review-1.md`. The deployed implementation is
`98eb121448edd957f8494df9577b7c47dcbc158b` at
<https://kube-permission-evidence.sociobot.in/>. The final documentation commit
is the commit containing this handoff; product files are unchanged after the
implementation SHA.

Kube Permission Evidence remains a read-only Rust CLI for Kubernetes operators
preparing audits or permission changes. Its first action is a one-click sample
on the site or `kpe demo` in a terminal.

## What changed

- Signed packets now reject unknown JSON fields at every authenticated packet
  level. Access-matrix objects also reject misspelled or unknown fields.
- `kpe verify` can require an approved public key or SHA-256 signer
  fingerprint. Its output distinguishes content integrity from signer trust.
- `kpe keygen` can write a public-key file for separate delivery.
- `kpe demo` writes a realistic four-check packet and bundled inputs to a new
  directory without kubectl, a cluster, or persistent application state.
- `/demo/` is a real one-click browser sample. It has a persistent sample-data
  label, Reset demo, and Start for real. Demo selection uses only
  `sessionStorage` key `demo:kpe:selected-case`.
- `.factory/claims.json` declares 34 public claims. Each has one matching,
  outcome-based `@claim:<id>` browser test.
- The landing page now states the job, audience, first action, and three facts
  before scrolling. Metaphor headings and unclear wording were removed.
- Root, demo, privacy, terms, and 404 pages have route-specific titles,
  metadata, shared navigation/footer, product art, and Param Factory credit.
- Unknown URLs now return the designed 404 page with HTTP 404.
- README now lists every kubectl call and documents demo, trust, package, and
  clean verification workflows.
- The service worker caches the complete site and demo for offline use while
  excluding license-bearing URLs.
- The catalog description is verb-first and 100 characters. It is copied to
  `/work/.evidence/catalog-description.txt`.

## Finding disposition

All 10 current findings are closed:

1. Unknown signed JSON fields: rejected by strict deserialization tests.
2. Trusted signer enforcement: public-key and fingerprint match/mismatch paths
   pass in source and installed-package tests.
3. Matrix typos: rejected before evaluation.
4. Demo contract: CLI and browser demos are present, isolated, resettable, and
   populated.
5. Claims contract: all 34 manifest commands pass independently.
6. Plain words: first-screen structure and `.factory/copy-audit.md` pass.
7. Missing route: designed page returns HTTP 404.
8. Metadata: route titles, canonical links, share image, and touch icon pass.
9. Site structure: shared header, footer, legal links, attribution, and build
   identifier are present on every page.
10. Collector documentation: all safe kubectl calls and flag behavior are
    listed.

Earlier repair findings remain covered: `resourceNames` semantics, wildcard
subresources, format/type gates, cache headers, mobile targets, service-worker
updates, license-verdict token binding, and license URL cache exclusion.

## Verification

The following passed from clean detached checkout
`/tmp/kpe-repair3-final.Xs80Qf`:

```text
npm ci                                      PASS — 0 vulnerabilities
npm audit --audit-level=high                PASS
npm test                                    PASS — 18 Rust + 47 browser tests
npm run build                               PASS — release CLI + dist/site
cargo test --doc                            PASS — 1
cargo package --locked --allow-dirty        PASS — 25 files
all 34 claims.json commands independently   PASS
clean packaged cargo install                PASS — kpe 0.1.0
installed kpe demo                          PASS — 4 checks, 2 allowed, 2 denied
installed trusted signer match/mismatch     PASS
factory verify-url, local and live          PASS
```

The production site ships 3.06 KiB gzip JavaScript and 4.58 KiB gzip CSS.
Local Lighthouse scored 100 in Performance, Accessibility, Best Practices,
and SEO; LCP was 1.51 s, CLS 0, and TBT 0 ms.

Fresh live desktop and iPhone-size contexts verified the job, audience, and
sample action before scrolling; populated demo output; persistent sample
label; reload; reset; exit; unchanged real-data sentinel; same-origin-only demo
requests; keyboard use; focus; reduced motion; offline home and demo; no
horizontal overflow; and zero unexpected console errors. Axe found zero
serious or critical issues on every route. The one console 404 entry from the
intentional missing-page navigation was classified as expected.

Live Lighthouse scored 100 in Performance, Accessibility, Best Practices, and
SEO. LCP was 1.36 s, CLS 0, and TBT 41 ms. Screenshots, verifier output, and the
Lighthouse report are under `/work/.evidence/live-repair-3/`.

## Repeat the checks

```sh
npm ci
npm audit --audit-level=high
npm test
npm run build
cargo test --doc
cargo package --locked --allow-dirty
```

Run a single public claim with, for example:

```sh
npm run test:claim -- --grep @claim:signature-trust
```

Install the staged package into an empty Cargo root and run `kpe --help`,
`kpe demo`, a signed `kpe report`, and `kpe verify` with matching and different
trusted keys.

## Deployment and known gaps

The implementation was pushed to `origin/main` and deployed successfully with
the product's existing static deployment. The custom domain serves the new
hashed assets and all expected security headers. Root, demo, privacy, and terms
return 200; a missing URL returns 404.

Field Kit sales remain honestly paused, as before. No checkout or paid mock is
shown. Publishing the ready Cargo package remains a factory release step; no
registry credentials were used. There are no known product defects from this
work order.

## Verification 4 — PASS

Independent verification on 2026-09-05 reviewed implementation
`98eb121448edd957f8494df9577b7c47dcbc158b` and documentation
`abe513f8391b92fc94f24d28b9891dab0257594d`. It found zero findings and zero
untested claims.

From a fresh detached checkout, `npm ci`, audit, `npm test`, `npm run build`,
doctest, and `cargo package` passed. All 34 declared claim commands were run
individually and passed. A packaged crate was installed into a clean consumer
root; its demo, signed report, trusted-signer match/mismatch, invalid input,
empty-matrix, and missing-kubectl paths passed.

Fresh live desktop and 390 px contexts confirmed the job, audience, and
sample-first action before scrolling; populated isolated demo; persistent demo
label; reset; exit; real-storage protection; keyboard focus; reduced motion;
offline demo; privacy; legal routes; links; and designed 404 response. Axe had
zero serious or critical issues. Live Lighthouse was 100 in all four
categories on retry. The live generated artifacts matched the implementation
build byte-for-byte.

See `.factory/verification-4.md` for the full evidence and prior-finding
disposition. The remaining external step is Cargo registry publishing; sales
remain paused.
