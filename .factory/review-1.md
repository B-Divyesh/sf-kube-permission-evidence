# Review 1 — explain Kubernetes access as audit evidence

## Verdict

**FAIL.** The review found **10 findings**: 5 high and 5 medium. There are
**34 untested public claim groups**. PASS requires zero findings and zero
untested claims.

- Work order: `kube-permission-evidence-review-1`
- Reviewed: 2026-09-05 UTC
- Live URL: <https://kube-permission-evidence.sociobot.in/>
- Implementation candidate: `aed9592164633ab4ef8f7ab9e5da082368f4ac7e`
- Documentation head before this review: `3c058becd13b80c2d105ed8b89df73414af521ea`
- Reason for different SHAs: commits after `aed9592` change only factory reports.
  The live root, privacy page, terms page, and service worker match a fresh
  `aed9592` build byte-for-byte.

## First screen before scrolling

- Job shown: trace a Kubernetes permission to the binding, role, and rule that
  grants it.
- Intended audience from the brief: Kubernetes operators preparing audits or
  permission changes.
- First action shown on desktop and phone: **Install the CLI**.

The headline names the job. The supporting sentence does not name the
audience, and the first action is not the required sample action.

## Findings

### F-01 — High — added JSON fields are not authenticated

The installed `kpe verify` accepts added fields at the report, check, grant,
and signature levels. Every modified file differed from the signed file, but
all four commands returned exit 0 and `{"valid":true}`. Changing a recognized
field correctly returned exit 1 with `content digest does not match`.

This is the first unresolved finding from verification 3. The parser drops
unknown fields before it recomputes the digest and signature.

### F-02 — High — verification cannot require a trusted signer

`kpe verify --help` offers only the packet path and `--json`. It has no
expected public key, fingerprint, or keyring option. Two packets signed by two
different freshly generated keys both returned `VALID` with exit 0.

The command proves internal signature consistency. It does not prove that an
auditor-approved operator signed the packet. This is the second unresolved
finding from verification 3.

### F-03 — High — misspelled matrix fields can change an access verdict

Matrix objects do not reject unknown fields. A check containing
`"apiGruop":"apps"` was accepted. The CLI silently changed the request to the
core API group and reported it allowed against a core `deployments` rule. The
same check with the correct `"apiGroup":"apps"` was denied.

An evidence tool must reject an unrecognized question field. Silent fallback
can produce both false grants and false denials.

### F-04 — High — the required CLI sample demo is absent

The live first screen has no **Try it with sample data** action. `/demo` serves
the normal landing page and normal title. There is no persistent **Demo —
sample data, nothing is saved** label, **Reset demo**, or **Start for real**.
The installed binary rejects both `kpe demo` and `kpe --demo` with exit 2.
`.factory/demo.md` is absent, and the landing terminal is static HTML rather
than a recording of the real binary.

The existing browser specimen is populated and realistic enough to show an
allowed, denied, and uncertain result. Its controls made no local/session
storage writes and contacted no external origin, so the exercised preview did
not change real data. It still does not satisfy the CLI demo contract.

### F-05 — High — public claims have no required claim evidence

`.factory/claims.json` is missing and no test contains an `@claim:` tag.
Therefore there are no claim commands to run from the clean checkout. The 34
public claim groups listed below have no one-to-one sandbox command, including
security, privacy, offline, output, exit-code, and RBAC-correctness claims.

Passing general tests does not meet the required claim contract. Each public
claim needs one declared test that observes the promised result.

### F-06 — Medium — the first screen and section copy break the plain-words contract

The first screen does not name Kubernetes operators. It leads with install,
not the sample. The site also uses metaphor or mood headings such as “Field
note,” “field key,” “Collect the habitat,” “Live specimen,” “Run it where the
evidence lives,” and “Leave with an evidence trail.” `.factory/copy-audit.md`
is absent.

Replace these with job, input, result, privacy, and limitation headings.

### F-07 — Medium — there is no real 404 response or page

`/missing-review-path` and `/404.html` return HTTP 200 and render the landing
page. The repository contains no designed 404 page or response override. This
is not a deliberate HTTP 404; it is a missing required error route.

### F-08 — Medium — required route metadata is missing

Root, privacy, terms, `/demo`, and the fallback page have no canonical link,
Open Graph metadata, Twitter card metadata, or Apple touch icon. There is no
1200×630 share image. `/demo` also keeps the landing title instead of setting
`Demo — Kube Permission Evidence`. The sitemap cannot list the required demo
or 404 routes because they do not exist.

### F-09 — Medium — legal pages and the footer do not use the required site skeleton

The privacy and terms routes have no footer. Their header contains only the
home link and no consistent navigation. The landing footer omits “Built by
Param Factory” and a build identifier. External GitHub links do not say they
open an external site.

### F-10 — Medium — the documented collector command boundary is incomplete

README says the collector runs “only these calls” and lists four `get` calls
plus `kubectl version -o json`. A controlled installed-binary run also invoked
`kubectl version --client -o json`, and, when context was omitted,
`kubectl config current-context`. These calls are read-only, but the exclusive
public list is false and has no claim test.

## Public claim inventory without declared tests

The count below groups repeated wording on the site, legal pages, CLI help,
and README by one observable promise. All 34 groups lack a claims entry.

1. Produces an auditor-readable point-in-time RBAC proof.
2. Collects only the stated RBAC objects and commands.
3. Evaluates a bounded subject and resource matrix.
4. Records every granting path.
5. Makes no cluster mutations.
6. Does not read or retain kubeconfig tokens.
7. Installs no agent or operator.
8. Sends no telemetry.
9. Uses no hosted cluster connection.
10. Runs reports offline without a cluster or kubectl.
11. Writes Markdown and JSON companions.
12. Sends complete JSON to stdout without writing files.
13. Uses the documented exit 0, exit 1, and exit 3 behavior.
14. Accepts the documented user, group, and service-account forms.
15. Derives standard Kubernetes service-account groups.
16. Evaluates RoleBinding to Role grants.
17. Constrains RoleBinding to ClusterRole grants by namespace.
18. Evaluates ClusterRoleBinding grants across namespaces.
19. Handles exact and wildcard verbs, API groups, and resources.
20. Handles subresources, including `*/subresource`.
21. Handles `resourceNames` and list/watch field selectors correctly.
22. Handles non-resource URL patterns.
23. Marks aggregation-rule results uncertain.
24. Records Kubernetes server version behavior.
25. States the excluded authorizers and identity-provider limits.
26. Creates reusable snapshots without credentials.
27. Creates owner-only signing keys and refuses overwrite.
28. Signs and verifies digest, signature, and public-key data.
29. Works with a least-privilege kubeconfig.
30. Uses fictional browser sample data without upload or cluster contact.
31. Loads without analytics, trackers, CDN fonts, or third-party scripts.
32. Keeps the guide and browser preview working offline.
33. Stores and verifies license data only as described and at most daily.
34. Accepts no payment while Field Kit sales are paused.

## Earlier finding disposition

| Earlier finding | Current evidence | Disposition |
| --- | --- | --- |
| Named top-level create and list/watch `resourceNames` semantics | Rust and release-binary regression tests pass | Resolved |
| Rust formatting and TypeScript check | Clean `npm test` passes both | Resolved |
| Cache and response security policy | Live headers and immutable assets pass | Resolved |
| Mobile targets under 44 px | Fresh 390 px scan found none | Resolved |
| Service-worker update behavior | v3 claims immediately; live offline reload passes | Resolved |
| Kubernetes `*/subresource` matching | Unit, matrix, and release-binary tests pass | Resolved |
| Unavailable paid checkout | Site says sales are paused and exposes no checkout | Resolved by honest removal |
| License verdict reused for another token | Browser regression test passes | Resolved |
| License query stored in Cache Storage | Browser regression test passes | Resolved |
| Unknown signed fields accepted | Fresh packaged reproduction returns exit 0 | Open, F-01 |
| No trusted-signer check | Fresh two-key reproduction returns exit 0 twice | Open, F-02 |

## Clean checkout, build, package, and CLI evidence

The clean detached worktree at the implementation candidate remained clean.

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 24 packages, 0 vulnerabilities |
| `npm audit --audit-level=high` | PASS — 0 vulnerabilities |
| `npm test` | PASS — typecheck, format, Clippy, 14 Rust tests, 14 Chromium tests |
| `npm run build` | PASS — release binary and `dist/site/` |
| `cargo test --doc` | PASS — 1 doctest |
| `cargo package --locked --allow-dirty` | PASS — 25 files, 37.1 KiB compressed |
| Clean packaged `cargo install --locked --path ...` | PASS — `kpe 0.1.0` |
| Documented offline report | PASS — 2 allowed, 2 denied, Markdown and JSON |
| Key generation | PASS — 32-byte seed file mode 0600 |
| Unchanged signed packet | PASS — exit 0 |
| Recognized-field tamper | PASS — exit 1 |
| Unknown-field tamper | FAIL — exit 0 at four object levels |
| Expected signer enforcement | FAIL — option absent |
| `--fail-on denied` and `--fail-on allowed` | PASS — exit 3 |
| Empty matrix | PASS — valid 0/0/0 report |
| Invalid subject, malformed JSON, blank verb, mixed targets | PASS — useful exit 1 errors |
| Conflicting output modes | PASS — exit 2 |
| Missing kubectl, missing input, unwritable output | PASS — useful exit 1 errors |
| Existing key and unsigned report | PASS — safe exit 1 errors |
| Controlled collector | PASS — read-only calls only; F-10 documents extra calls |
| CLI demo | FAIL — command absent |

No real Kubernetes cluster was available. The installed collector was run
against a controlled kubectl process that returned Kubernetes-shaped data and
recorded every invocation. Context and kubeconfig arguments were passed
through, and the resulting snapshot contained no token or credential text.

## Live browser, accessibility, privacy, and offline evidence

Fresh isolated Chromium contexts were used at 1440×900 and 390×844.

- Root, privacy, and terms have one h1, a main landmark, `lang=en`, route
  titles, labeled controls, image alt text, and no console or page errors.
- Playwright axe 4.10.2 found no violations on the exercised root state and no
  serious or critical violations on root, privacy, terms, `/demo`, or the
  fallback page.
- The factory URL verifier passed in 548 ms. The standalone axe CLI could not
  pair the worker's ChromeDriver 152 with preinstalled Chromium 145; the
  required equivalent Playwright axe run completed instead.
- Keyboard Tab reaches the skip link first. Focus uses a visible 3 px outline.
  Select, clear, copy, details, links, and form controls are keyboard operable.
- No visible target was smaller than 44×44 px. There was no page overflow at
  desktop or phone widths.
- Reduced motion changes hero animation and button transition duration to
  0.01 ms and disables smooth scrolling.
- Allowed, denied, uncertain, and empty browser specimen states work.
- A clean first load creates no local or session storage and contacts only the
  product origin.
- Invalid license recovery returns a clear locked state. The verifier response
  is origin-scoped and non-cacheable. The isolated browser context was deleted
  after the run.
- Service-worker control, update activation, explicit offline notice, offline
  reload, and offline specimen selection pass.
- Internal downloads and legal links return 200. Both public GitHub links
  resolve to 200.
- Privacy deletion is local: clearing site storage removes saved license data.
  There is no product account or product backend to test for data export or
  tenant deletion.
- Backend tenant isolation, restart persistence, health, and 429 behavior do
  not apply to this static site plus local CLI.

Lighthouse simulated mobile results are Performance 100, Accessibility 100,
Best Practices 100, and SEO 100. FCP is 1.01 s, LCP 1.37 s, TBT 41 ms, CLS 0,
and total transfer is about 103 KiB. Built JavaScript is 5,986 bytes, CSS is
14,183 bytes, the hero is 93,322 bytes, and no fonts are downloaded.

## AI feature check

No AI feature is missing. The core job requires deterministic, inspectable
RBAC evaluation. A model-generated conclusion would weaken the evidence path.

## Required next work

1. Make signed JSON strict and add trusted-signer enforcement with regression
   tests on the installed release artifact.
2. Reject unknown matrix fields and test misspellings that would change a
   verdict.
3. Add the real CLI demo, live demo route, persistent demo controls, and demo
   documentation.
4. Add `.factory/claims.json` and one observable sandbox test per public claim;
   correct the collector command claim.
5. Replace metaphor copy, add the copy audit, complete route metadata and the
   shared skeleton, and ship a real 404 response and page.
