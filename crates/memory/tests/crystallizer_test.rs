use metaygn_memory::crystallizer::SkillCrystallizer;

fn seq(tools: &[&str]) -> Vec<String> {
    tools.iter().map(|s| s.to_string()).collect()
}

#[test]
fn observe_counts_patterns() {
    let mut sc = SkillCrystallizer::new(3);
    let tools = seq(&["Grep", "Read", "Edit"]);

    sc.observe(&tools);
    sc.observe(&tools);
    sc.observe(&tools);

    let crystallized = sc.crystallized();
    assert_eq!(crystallized.len(), 1);
    assert_eq!(crystallized[0].count, 3);
    assert_eq!(crystallized[0].tools, tools);
}

#[test]
fn threshold_filters_infrequent() {
    let mut sc = SkillCrystallizer::new(3);
    let tools = seq(&["Read", "Edit"]);

    sc.observe(&tools);
    sc.observe(&tools);

    // Only 2 observations, threshold is 3
    let crystallized = sc.crystallized();
    assert!(crystallized.is_empty());
    assert_eq!(sc.total_patterns(), 1);
}

#[test]
fn different_sequences_tracked_separately() {
    let mut sc = SkillCrystallizer::new(1);
    let seq_a = seq(&["Read", "Edit"]);
    let seq_b = seq(&["Grep", "Read"]);

    sc.observe(&seq_a);
    sc.observe(&seq_b);

    assert_eq!(sc.total_patterns(), 2);
    let crystallized = sc.crystallized();
    assert_eq!(crystallized.len(), 2);
}

#[test]
fn generate_produces_valid_markdown() {
    let mut sc = SkillCrystallizer::new(1);
    let tools = seq(&["Grep", "Read", "Edit"]);
    sc.observe(&tools);

    let crystallized = sc.crystallized();
    assert_eq!(crystallized.len(), 1);

    let md = SkillCrystallizer::generate_skill_md(crystallized[0]);

    // YAML front matter
    assert!(md.starts_with("---\n"), "should start with YAML front matter");
    assert!(md.contains("name: crystallized-"), "should contain name");
    assert!(
        md.contains("description: Auto-detected pattern"),
        "should contain description"
    );

    // Content
    assert!(md.contains("# Crystallized Pattern"), "should have heading");
    assert!(md.contains("1. Grep"), "should list tools numbered");
    assert!(md.contains("2. Read"), "should list tools numbered");
    assert!(md.contains("3. Edit"), "should list tools numbered");
}

#[test]
fn hash_is_deterministic() {
    let mut sc1 = SkillCrystallizer::new(1);
    let mut sc2 = SkillCrystallizer::new(1);
    let tools = seq(&["Grep", "Read", "Edit"]);

    sc1.observe(&tools);
    sc2.observe(&tools);

    let c1 = sc1.crystallized();
    let c2 = sc2.crystallized();

    assert_eq!(c1[0].hash, c2[0].hash, "same sequence should produce same hash");
    assert_eq!(c1[0].hash.len(), 64, "SHA-256 hex digest should be 64 chars");
}
