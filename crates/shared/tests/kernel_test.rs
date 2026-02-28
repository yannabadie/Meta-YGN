use metaygn_shared::kernel::{AlignmentRule, Kernel};

#[test]
fn kernel_hash_is_deterministic() {
    let k1 = Kernel::default();
    let k2 = Kernel::default();
    assert_eq!(k1.hash(), k2.hash());
}

#[test]
fn kernel_verify_passes_on_unmodified() {
    let k = Kernel::default();
    assert!(k.verify().is_ok());
}

#[test]
fn kernel_verify_fails_on_tampered() {
    let mut k = Kernel::default();
    k.rules_mut().push(AlignmentRule::Custom("sneaky".into()));
    let result = k.verify();
    assert!(result.is_err());
}

#[test]
fn kernel_default_has_5_rules() {
    let k = Kernel::default();
    assert_eq!(k.rules().len(), 5);
}
