use metaygn_verifiers::evidence::EvidencePack;

#[test]
fn empty_pack_verifies() {
    let pack = EvidencePack::new();
    assert!(pack.is_empty());
    assert_eq!(pack.len(), 0);
    assert!(pack.verify_chain().is_ok());
}

#[test]
fn single_entry_verifies() {
    let mut pack = EvidencePack::new();
    pack.append("test_event", serde_json::json!({"key": "value"}));
    assert_eq!(pack.len(), 1);
    assert!(!pack.is_empty());

    // First entry must have prev_hash = [0u8; 32]
    let entry = &pack.entries()[0];
    assert_eq!(entry.prev_hash, [0u8; 32]);

    assert!(pack.verify_chain().is_ok());
}

#[test]
fn chain_of_three_verifies() {
    let mut pack = EvidencePack::new();
    pack.append("event_a", serde_json::json!({"step": 1}));
    pack.append("event_b", serde_json::json!({"step": 2}));
    pack.append("event_c", serde_json::json!({"step": 3}));

    assert_eq!(pack.len(), 3);
    assert!(pack.verify_chain().is_ok());

    // Each entry's prev_hash should differ from its predecessor's prev_hash
    let entries = pack.entries();
    assert_ne!(entries[1].prev_hash, entries[0].prev_hash);
    assert_ne!(entries[2].prev_hash, entries[1].prev_hash);
}

#[test]
fn tampered_chain_fails() {
    let mut pack = EvidencePack::new();
    pack.append("event_a", serde_json::json!({"data": "original"}));
    pack.append("event_b", serde_json::json!({"data": "second"}));
    pack.append("event_c", serde_json::json!({"data": "third"}));

    assert!(pack.verify_chain().is_ok());

    // Tamper with the first entry's payload
    pack.entries_mut()[0].payload = serde_json::json!({"data": "tampered"});

    // Chain verification should now fail
    let result = pack.verify_chain();
    assert!(result.is_err());
}

#[test]
fn merkle_root_deterministic() {
    let mut pack = EvidencePack::new();
    for i in 0..5 {
        pack.append("event", serde_json::json!({"index": i}));
    }

    // Computing the Merkle root multiple times must yield the same result
    let root1 = pack.merkle_root();
    let root2 = pack.merkle_root();
    let root3 = pack.merkle_root();

    assert_eq!(root1, root2);
    assert_eq!(root2, root3);

    // Root must not be the zero hash (pack is non-empty)
    assert_ne!(root1, [0u8; 32]);
}

#[test]
fn merkle_root_changes_on_modification() {
    let mut pack = EvidencePack::new();
    pack.append("event_a", serde_json::json!({"x": 1}));
    pack.append("event_b", serde_json::json!({"x": 2}));

    let root_before = pack.merkle_root();

    // Add another entry — the Merkle root must change
    pack.append("event_c", serde_json::json!({"x": 3}));
    let root_after = pack.merkle_root();

    assert_ne!(root_before, root_after);
}

#[test]
fn sign_and_verify() {
    let mut pack = EvidencePack::with_signing();
    pack.append("signed_event", serde_json::json!({"important": true}));

    // Sign the last entry
    let signature = pack.sign_last().expect("signing key is present");

    // Get the public key from the pack
    let public_key = pack.public_key().expect("signing key is present");

    // Verify the signature
    assert!(pack.verify_signature(&signature, &public_key));

    // Tamper with the signature — verification should fail
    let mut bad_sig = signature;
    bad_sig[0] ^= 0xFF;
    assert!(!pack.verify_signature(&bad_sig, &public_key));
}
