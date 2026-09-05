# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/) and
this project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- Reject unknown fields at every signed evidence boundary and in access
  matrices.
- Verify packets against a separately delivered public key or signer
  fingerprint.
- Document every read-only kubectl command used during collection.
- Add complete route metadata, shared navigation and footers, and a real 404
  response configuration.

### Added

- Add `kpe demo` with bundled input and isolated temporary output.
- Add a one-click browser demo with reset and exit controls.
- Add one outcome-based regression test for each declared public claim.

## [0.1.0] - 2026-08-28

### Fixed

- Matched Kubernetes RBAC rules of the form `*/subresource` without allowing
  parent resources or other subresources.
- Bound cached license verdicts to the exact verified token and prevented
  license-bearing return URLs from entering service-worker Cache Storage.
- Paused the not-yet-enabled Field Kit offer instead of linking to an
  unavailable checkout or promising unpublished downloads.
- Prevented `resourceNames` rules from falsely granting top-level `create` or
  `deletecollection`, and required a matching `metadata.name` field selector
  for named `list` and `watch` evidence.
- Added release-gated TypeScript checks, Azure Static Web Apps security/cache
  policy, immediate service-worker activation, and 44 px touch targets.

### Added

- Read-only live RBAC collection and reusable offline snapshots.
- User, group, and service-account access-matrix evaluation.
- Auditor-readable Markdown and structured JSON evidence packets.
- Optional Ed25519 packet signing and offline verification.
- Static documentation, live evidence walkthrough, and one-time license flow.
