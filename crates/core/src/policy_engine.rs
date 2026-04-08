//! Declarative policy engine using Datalog (crepe crate).
//!
//! Replaces hand-coded safety rules with composable Datalog rules
//! that operate on a sliding window of symbolic action states.
//! Inspired by PCAS (arxiv 2602.16708).

use crepe::crepe;

use crate::sequence_monitor::{ActionState, ActionType, TargetType};

// Action type IDs (must be Copy for crepe)
const ACT_WRITE: u32 = 1;
const ACT_DELETE: u32 = 2;
const ACT_NETWORK_READ: u32 = 4;
const ACT_GIT_PUSH: u32 = 5;
const ACT_ERROR: u32 = 6;

// Target type IDs
const TGT_TEST_FILE: u32 = 1;
const TGT_SENSITIVE_PATH: u32 = 2;

/// A violation detected by the policy engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PolicyViolation {
    pub rule: String,
    pub description: String,
    pub step1: usize,
    pub step2: usize,
}

crepe! {
    @input
    struct Action(usize, u32, u32); // (step, action_type_id, target_type_id)

    @output
    struct NetSensWrite(usize, usize);

    @output
    struct DelForcePush(usize, usize);

    @output
    struct ErrTestMod(usize, usize);

    // Rule 1: network read followed by sensitive write
    NetSensWrite(s1, s2) <-
        Action(s1, a1, _t1), (a1 == ACT_NETWORK_READ),
        Action(s2, a2, t2), (a2 == ACT_WRITE), (t2 == TGT_SENSITIVE_PATH),
        (s1 < s2);

    // Rule 2: delete followed by git push
    DelForcePush(s1, s2) <-
        Action(s1, a1, _t1), (a1 == ACT_DELETE),
        Action(s2, a2, _t2), (a2 == ACT_GIT_PUSH),
        (s1 < s2);

    // Rule 3: error followed by test file write
    ErrTestMod(s1, s2) <-
        Action(s1, a1, _t1), (a1 == ACT_ERROR),
        Action(s2, a2, t2), (a2 == ACT_WRITE), (t2 == TGT_TEST_FILE),
        (s1 < s2);
}

fn action_type_id(at: ActionType) -> u32 {
    match at {
        ActionType::Read => 0,
        ActionType::Write => ACT_WRITE,
        ActionType::Delete => ACT_DELETE,
        ActionType::Execute => 3,
        ActionType::NetworkRead => ACT_NETWORK_READ,
        ActionType::GitPush => ACT_GIT_PUSH,
        ActionType::Error => ACT_ERROR,
        ActionType::Privilege => 7,
        ActionType::Unknown => 8,
    }
}

fn target_type_id(tt: TargetType) -> u32 {
    match tt {
        TargetType::File => 0,
        TargetType::TestFile => TGT_TEST_FILE,
        TargetType::SensitivePath => TGT_SENSITIVE_PATH,
        TargetType::Url => 3,
        TargetType::Remote => 4,
        TargetType::Unknown => 5,
    }
}

/// Evaluate safety policies against a window of actions.
pub fn evaluate(actions: &[(usize, &ActionState)]) -> Vec<PolicyViolation> {
    let mut runtime = Crepe::new();

    for &(step, action) in actions {
        runtime.extend([Action(
            step,
            action_type_id(action.action_type),
            target_type_id(action.target_type),
        )]);
    }

    let (net_writes, del_pushes, err_tests) = runtime.run();

    let mut violations = Vec::new();

    for NetSensWrite(s1, s2) in net_writes {
        violations.push(PolicyViolation {
            rule: "network_then_sensitive_write".into(),
            description: format!("Network read at step {s1} then sensitive write at step {s2}"),
            step1: s1,
            step2: s2,
        });
    }

    for DelForcePush(s1, s2) in del_pushes {
        violations.push(PolicyViolation {
            rule: "delete_then_force_push".into(),
            description: format!("Delete at step {s1} then force push at step {s2}"),
            step1: s1,
            step2: s2,
        });
    }

    for ErrTestMod(s1, s2) in err_tests {
        violations.push(PolicyViolation {
            rule: "errors_then_test_modify".into(),
            description: format!("Error at step {s1} then test modify at step {s2}"),
            step1: s1,
            step2: s2,
        });
    }

    // Deduplicate: keep first per rule
    let mut seen = std::collections::HashSet::new();
    violations.retain(|v| seen.insert(v.rule.clone()));

    violations
}
