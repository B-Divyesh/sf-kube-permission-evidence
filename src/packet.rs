use crate::model::{EvidenceReport, PacketSignature};
use anyhow::{Context, Result, bail};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn generate_key(path: &Path) -> Result<()> {
    let key = SigningKey::generate(&mut OsRng);
    let encoded = hex::encode(key.to_bytes());
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .with_context(|| {
                format!(
                    "could not create {} (refusing to overwrite)",
                    path.display()
                )
            })?;
        writeln!(file, "{encoded}")?;
    }
    #[cfg(not(unix))]
    {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .with_context(|| {
                format!(
                    "could not create {} (refusing to overwrite)",
                    path.display()
                )
            })?;
        writeln!(file, "{encoded}")?;
    }
    Ok(())
}

pub fn sign(report: &mut EvidenceReport, key_path: &Path) -> Result<()> {
    report.signature = None;
    let seed_hex = fs::read_to_string(key_path)
        .with_context(|| format!("could not read signing key {}", key_path.display()))?;
    let seed: [u8; 32] = hex::decode(seed_hex.trim())
        .context("signing key is not hex")?
        .try_into()
        .map_err(|_| anyhow::anyhow!("signing key must contain exactly 32 bytes"))?;
    let key = SigningKey::from_bytes(&seed);
    let content = serde_json::to_vec(report)?;
    let signature = key.sign(&content);
    report.signature = Some(PacketSignature {
        algorithm: "Ed25519".into(),
        public_key: STANDARD.encode(key.verifying_key().to_bytes()),
        signature: STANDARD.encode(signature.to_bytes()),
        content_sha256: hex::encode(Sha256::digest(&content)),
    });
    Ok(())
}

pub fn verify(report: &EvidenceReport) -> Result<()> {
    let packet_signature = report.signature.as_ref().context("packet is not signed")?;
    if packet_signature.algorithm != "Ed25519" {
        bail!(
            "unsupported signature algorithm: {}",
            packet_signature.algorithm
        );
    }
    let mut unsigned = report.clone();
    unsigned.signature = None;
    let content = serde_json::to_vec(&unsigned)?;
    let digest = hex::encode(Sha256::digest(&content));
    if digest != packet_signature.content_sha256 {
        bail!("content digest does not match; packet was changed");
    }
    let public_bytes: [u8; 32] = STANDARD
        .decode(&packet_signature.public_key)
        .context("invalid public key encoding")?
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid public key length"))?;
    let signature_bytes: [u8; 64] = STANDARD
        .decode(&packet_signature.signature)
        .context("invalid signature encoding")?
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid signature length"))?;
    let public_key = VerifyingKey::from_bytes(&public_bytes).context("invalid public key")?;
    public_key
        .verify(&content, &Signature::from_bytes(&signature_bytes))
        .context("signature verification failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn report() -> EvidenceReport {
        EvidenceReport {
            schema_version: "kpe.evidence/v1".into(),
            generated_at: "now".into(),
            tool_version: "test".into(),
            subject: SubjectIdentity {
                kind: "User".into(),
                name: "alice".into(),
                namespace: None,
            },
            evaluated_groups: vec![],
            source: EvidenceSource {
                collected_at: "then".into(),
                context: "test".into(),
                server_version: "v1".into(),
                snapshot_sha256: "00".into(),
                role_count: 0,
                cluster_role_count: 0,
                role_binding_count: 0,
                cluster_role_binding_count: 0,
            },
            summary: EvidenceSummary {
                total: 0,
                allowed: 0,
                denied: 0,
                uncertain: 0,
            },
            checks: vec![],
            limitations: vec![],
            signature: None,
        }
    }

    #[test]
    fn signed_packet_verifies_and_tampering_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("key");
        generate_key(&path).unwrap();
        let mut packet = report();
        sign(&mut packet, &path).unwrap();
        verify(&packet).unwrap();
        packet.subject.name = "mallory".into();
        assert!(verify(&packet).is_err());
    }
}
