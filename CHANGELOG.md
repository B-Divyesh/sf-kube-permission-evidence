# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/) and
this project uses [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-08-28

### Fixed

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
