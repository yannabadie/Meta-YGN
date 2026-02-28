//! Heuristic evolver — Layer 0 meta-metacognition.
//!
//! This module makes MetaYGN's risk classification and strategy selection
//! heuristics **evolvable** over time. It uses statistical learning from
//! session outcomes (French OPENSAGE variant) rather than LLM-driven mutation.
//!
//! The system tracks which risk markers and strategies lead to good/bad
//! outcomes, then adjusts scores using a lightweight evolutionary algorithm
//! with multi-objective fitness (AlphaEvolve-inspired).

pub mod evolver;
pub mod fitness;
