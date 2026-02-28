use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Alignment rules that govern agent behavior.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlignmentRule {
    RequireApprovalForDestructive,
    NeverExposeSecrets,
    EvidenceRequiredForStrongClaims,
    EscalateOnLowConfidence { threshold: f32 },
    PreserveUserIntent,
    Custom(String),
}

impl fmt::Display for AlignmentRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlignmentRule::RequireApprovalForDestructive => {
                write!(f, "RequireApprovalForDestructive")
            }
            AlignmentRule::NeverExposeSecrets => write!(f, "NeverExposeSecrets"),
            AlignmentRule::EvidenceRequiredForStrongClaims => {
                write!(f, "EvidenceRequiredForStrongClaims")
            }
            AlignmentRule::EscalateOnLowConfidence { threshold } => {
                write!(f, "EscalateOnLowConfidence(threshold={threshold})")
            }
            AlignmentRule::PreserveUserIntent => write!(f, "PreserveUserIntent"),
            AlignmentRule::Custom(s) => write!(f, "Custom({s})"),
        }
    }
}

/// Error type for KERNEL integrity violations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum KernelError {
    #[error("KERNEL integrity violation: expected {expected}, actual {actual}")]
    IntegrityViolation { expected: String, actual: String },
}

/// The KERNEL ensures alignment rules remain tamper-evident.
///
/// On construction it computes a SHA-256 hash of the serialized rule set and
/// stores it as `boot_hash`. The `verify()` method recomputes the hash and
/// compares it to the stored value, detecting any post-boot mutations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kernel {
    rules: Vec<AlignmentRule>,
    boot_hash: [u8; 32],
}

impl Kernel {
    /// Create a new Kernel with the given rules.
    /// Computes and stores the SHA-256 hash of the serialized rules.
    pub fn new(rules: Vec<AlignmentRule>) -> Self {
        let boot_hash = Self::compute_hash(&rules);
        Self { rules, boot_hash }
    }

    /// Returns the boot hash.
    pub fn hash(&self) -> [u8; 32] {
        self.boot_hash
    }

    /// Returns a reference to the alignment rules.
    pub fn rules(&self) -> &[AlignmentRule] {
        &self.rules
    }

    /// Returns a mutable reference to the alignment rules.
    ///
    /// **Warning**: Mutating rules after construction will cause `verify()` to
    /// fail, which is the intended tamper-detection mechanism.
    pub fn rules_mut(&mut self) -> &mut Vec<AlignmentRule> {
        &mut self.rules
    }

    /// Recomputes the hash of the current rules and compares it against the
    /// stored boot hash. Returns `Ok(())` if they match, or
    /// `Err(KernelError::IntegrityViolation)` if the rules have been tampered with.
    pub fn verify(&self) -> Result<(), KernelError> {
        let current_hash = Self::compute_hash(&self.rules);
        if current_hash == self.boot_hash {
            Ok(())
        } else {
            Err(KernelError::IntegrityViolation {
                expected: hex::encode(self.boot_hash),
                actual: hex::encode(current_hash),
            })
        }
    }

    /// Internal helper: serialize rules to JSON and compute SHA-256.
    fn compute_hash(rules: &[AlignmentRule]) -> [u8; 32] {
        let serialized =
            serde_json::to_string(rules).expect("AlignmentRule serialization must not fail");
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let result = hasher.finalize();
        result.into()
    }
}

impl Default for Kernel {
    /// Creates a Kernel with the 5 standard alignment rules.
    fn default() -> Self {
        let rules = vec![
            AlignmentRule::RequireApprovalForDestructive,
            AlignmentRule::NeverExposeSecrets,
            AlignmentRule::EvidenceRequiredForStrongClaims,
            AlignmentRule::EscalateOnLowConfidence { threshold: 0.3 },
            AlignmentRule::PreserveUserIntent,
        ];
        Self::new(rules)
    }
}

/// Helper to hex-encode bytes (avoids pulling in the `hex` crate).
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}
