use chrono::{DateTime, Utc};
use ed25519_dalek::{Signer, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum EvidenceError {
    #[error("hash chain broken at index {index}: expected {expected}, got {actual}")]
    BrokenChain {
        index: usize,
        expected: String,
        actual: String,
    },
}

// ---------------------------------------------------------------------------
// EvidenceEntry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub prev_hash: [u8; 32],
}

impl EvidenceEntry {
    /// SHA-256 hash of the entry's canonical (JSON) serialized form.
    fn hash(&self) -> [u8; 32] {
        let bytes = serde_json::to_vec(self).expect("EvidenceEntry is always serializable");
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        hasher.finalize().into()
    }
}

// ---------------------------------------------------------------------------
// EvidencePack
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct EvidencePack {
    entries: Vec<EvidenceEntry>,
    signing_key: Option<ed25519_dalek::SigningKey>,
}

impl EvidencePack {
    /// Create a new evidence pack without signing capability.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            signing_key: None,
        }
    }

    /// Create a new evidence pack with a randomly generated ed25519 key pair.
    pub fn with_signing() -> Self {
        let mut csprng = rand_core::OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        Self {
            entries: Vec::new(),
            signing_key: Some(signing_key),
        }
    }

    /// Append an entry to the pack. The hash chain is automatically maintained.
    pub fn append(&mut self, event_type: &str, payload: serde_json::Value) -> &EvidenceEntry {
        let prev_hash = match self.entries.last() {
            Some(prev) => prev.hash(),
            None => [0u8; 32],
        };

        let entry = EvidenceEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: event_type.to_string(),
            payload,
            prev_hash,
        };

        self.entries.push(entry);
        self.entries.last().unwrap()
    }

    /// Return a shared slice of all entries.
    pub fn entries(&self) -> &[EvidenceEntry] {
        &self.entries
    }

    /// Return a mutable slice of all entries (useful for testing tamper detection).
    pub fn entries_mut(&mut self) -> &mut [EvidenceEntry] {
        &mut self.entries
    }

    /// Number of entries in the pack.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the pack contains zero entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Verify every link in the hash chain.
    ///
    /// - The first entry must have `prev_hash == [0u8; 32]`.
    /// - Every subsequent entry's `prev_hash` must equal the SHA-256 hash of the
    ///   serialised previous entry.
    pub fn verify_chain(&self) -> Result<(), EvidenceError> {
        for (i, entry) in self.entries.iter().enumerate() {
            let expected = if i == 0 {
                [0u8; 32]
            } else {
                self.entries[i - 1].hash()
            };

            if entry.prev_hash != expected {
                return Err(EvidenceError::BrokenChain {
                    index: i,
                    expected: format!("{:x?}", expected),
                    actual: format!("{:x?}", entry.prev_hash),
                });
            }
        }
        Ok(())
    }

    /// Compute the Merkle tree root of all entries.
    ///
    /// - Leaf hashes = SHA-256 of each entry's serialized form.
    /// - If odd number of leaves, the last leaf is duplicated.
    /// - Returns `[0u8; 32]` for an empty pack.
    pub fn merkle_root(&self) -> [u8; 32] {
        if self.entries.is_empty() {
            return [0u8; 32];
        }

        let mut layer: Vec<[u8; 32]> = self.entries.iter().map(|e| e.hash()).collect();

        while layer.len() > 1 {
            // Duplicate last if odd
            if layer.len() % 2 != 0 {
                let last = *layer.last().unwrap();
                layer.push(last);
            }

            let mut next_layer = Vec::with_capacity(layer.len() / 2);
            for pair in layer.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(pair[0]);
                hasher.update(pair[1]);
                next_layer.push(hasher.finalize().into());
            }
            layer = next_layer;
        }

        layer[0]
    }

    /// Sign the serialized last entry with the pack's ed25519 signing key.
    ///
    /// Returns `None` if there is no signing key or the pack is empty.
    pub fn sign_last(&self) -> Option<[u8; 64]> {
        let key = self.signing_key.as_ref()?;
        let last = self.entries.last()?;
        let bytes = serde_json::to_vec(last).expect("EvidenceEntry is always serializable");
        let signature = key.sign(&bytes);
        Some(signature.to_bytes())
    }

    /// Verify an ed25519 signature of the last entry against the given public key.
    ///
    /// Returns `false` if the pack is empty or the signature / key is invalid.
    pub fn verify_signature(&self, signature: &[u8; 64], public_key: &[u8; 32]) -> bool {
        let Ok(verifying_key) = VerifyingKey::from_bytes(public_key) else {
            return false;
        };
        let Some(last) = self.entries.last() else {
            return false;
        };
        let bytes = serde_json::to_vec(last).expect("EvidenceEntry is always serializable");
        let sig = ed25519_dalek::Signature::from_bytes(signature);
        verifying_key.verify(&bytes, &sig).is_ok()
    }

    /// Return the public key bytes if a signing key is present.
    pub fn public_key(&self) -> Option<[u8; 32]> {
        self.signing_key
            .as_ref()
            .map(|k| k.verifying_key().to_bytes())
    }
}

impl Default for EvidencePack {
    fn default() -> Self {
        Self::new()
    }
}
