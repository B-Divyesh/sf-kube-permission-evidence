# Visual thesis — the authorization herbarium

## Direction and rationale

Kube Permission Evidence is presented as a **botanical field guide**. Kubernetes
authorization is usually shown as an abstract graph; a field guide makes the
same work feel patient, inspectable, and evidentiary. Subjects are roots,
bindings are labelled stems, roles are branching structures, and effective
permissions are the leaves a reviewer can identify. The visual language is
calm enough for an audit packet but distinctive enough not to resemble another
cloud dashboard.

The site is deliberately single-mode, like a cream archival folio under warm
workbench light. Dark mode is not used because the paper-and-ink metaphor and
long-form documentation benefit from a stable print-adjacent treatment. The
background is explicitly painted in every route.

## Palette

| Token | Value | Role |
| --- | --- | --- |
| paper | `#F4F0E4` | page background |
| paper-deep | `#E7E0CF` | code fields and specimen mats |
| ink | `#17231C` | primary copy |
| moss | `#244D3B` | primary actions and headings |
| fern | `#4D6A53` | secondary botanical detail |
| clay | `#9B4D32` | seals, focus, and key evidence |
| ochre | `#B47B2A` | warnings and uncertainty |
| rule | `#B9B19E` | dividers and quiet outlines |
| muted | `#59665D` | secondary copy |
| success | `#276447` | confirmed access |
| danger | `#8A362E` | denied/invalid state |

All normal text combinations meet WCAG AA (4.5:1 or better). Status is always
written or paired with a symbol, never communicated with colour alone.

## Type

- Display and guide headings: Georgia, Cambria, `Times New Roman`, serif. The
  high-contrast letterforms recall specimen labels and remain system-local.
- Interface, body, and code: `ui-monospace`, SFMono-Regular, Consolas,
  `Liberation Mono`, monospace. The technical rhythm keeps commands and causal
  evidence visually honest.
- Scale: 14, 16, 18, 24, 36, and clamp(46–76) px; body never drops below 16 px.
  Prose is capped at 70 characters.

No fonts are downloaded, so the site has no font privacy or performance cost.

## Spacing and layout

An 8 px base rhythm uses 8, 16, 24, 32, 48, 64, 96, and 128 px intervals.
Hairline horizontal rules evoke ledger entries. Content groups rely on
proximity before outlines. Independent specimens (the three proof stages and
pricing) receive paper mats; documentation stays in a single reading column.
Desktop uses an asymmetric 7/5 field-guide spread. At 390 px it becomes one
column, omits decorative margin notes, and keeps commands horizontally
scrollable. All targets are at least 44 px.

## Interaction grammar

- Links gain a clay underline; buttons depress by 1 px like a hand stamp.
- Copy controls acknowledge with `Copied` in a polite live region.
- The demo evidence trail reveals the selected check as one continuous causal
  path; denied checks show the absence of a matching specimen.
- License verification is quiet and never blocks the free documentation.

## Motion policy

Initial specimen parts enter once from their physical origin (4–10 px upward,
180–280 ms). Readable copy stays fully opaque while it moves. State changes
use opacity and a short transform only when the content remains readable.
Nothing loops.
With `prefers-reduced-motion: reduce`, transitions and smooth scrolling become
instant; hierarchy, borders, and labels preserve every state without motion.

## Asset plan and provenance

- `site/public/specimen-map.webp`: original AI-generated landscape
  illustration, prompted on 2026-08-28 with the factory image deployment. It
  depicts a pressed fern whose root/stem/leaf structure explains the
  subject→binding→role→resource chain. No text, logos, people, or third-party
  assets. Generation metadata and the exact prompt are retained as provenance;
  the shipped WebP is optimised below 300 KB.
- `site/public/share-card.webp`: deterministic 1200×630 center crop of the
  original specimen map. Created locally with ImageMagick; no new source art.
- `site/public/apple-touch-icon.png`: deterministic 180×180 crop of the
  original fern. Created locally with ImageMagick; no new source art.
- Seal, arrows, and UI glyphs are hand-made with CSS or inline SVG primitives;
  no icon library is used.

Final generation prompt:

> Use case: scientific-educational. Asset type: landing page hero illustration
> for a Kubernetes authorization evidence CLI. Scene: an archival botanical
> field-guide plate on warm unbleached paper. Subject: one pressed fern-like
> specimen whose fine roots merge into a central stem, branch through several
> labelled-looking but completely blank archival tags, and terminate in a
> small set of distinct leaves; thin rust-red curator threads trace one causal
> route from root to leaf. Style: precise 19th-century botanical engraving
> crossed with modern technical ink drawing, tactile paper grain, restrained
> screenprint texture. Composition: landscape, specimen on the right two
> thirds, generous calm negative space on the left for HTML heading, subtle
> ledger grid and one circular evidence seal shape. Palette: deep forest green,
> moss, faded clay, ochre, charcoal ink, warm cream. Mood: trustworthy,
> forensic, quiet. Constraints: no words, no letters, no numbers, no logos, no
> Kubernetes logo, no people, no computers, no gradients, no glossy 3D, no
> watermark; flat readable silhouette and strong contrast.
