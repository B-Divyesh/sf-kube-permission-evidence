//! Core evaluator for Kube Permission Evidence.
//!
//! The public API is intentionally small: deserialize a [`Snapshot`] and
//! [`AccessMatrix`], parse a subject, then call [`evaluate`].
//!
//! ```
//! use kube_permission_evidence::{evaluate, parse_subject, AccessMatrix, Snapshot};
//!
//! let snapshot: Snapshot = serde_json::from_str(r#"{
//!   "schemaVersion":"kpe.snapshot/v1", "collectedAt":"2026-08-28T00:00:00Z",
//!   "collectorVersion":"0.1.0", "context":"fixture", "serverVersion":"v1.33.0",
//!   "roles":[], "clusterRoles":[], "roleBindings":[], "clusterRoleBindings":[]
//! }"#)?;
//! let matrix: AccessMatrix = serde_json::from_str(
//!   r#"{"checks":[{"verb":"get","resource":"secrets","namespace":"prod"}]}"#
//! )?;
//! let subject = parse_subject("user:alice@example.com")?;
//! let report = evaluate(&snapshot, &subject, &[], &matrix);
//! assert_eq!(report.summary.denied, 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod collector;
mod evaluator;
mod model;
pub mod packet;
pub mod render;

pub use evaluator::{evaluate, evaluated_groups, parse_subject};
pub use model::*;
