//! Tool forge — generates, executes, and caches ephemeral verification scripts.
//!
//! The forge uses **static templates** (not LLM generation) to produce small
//! verification tools that run inside the [`ProcessSandbox`].  Each generated
//! script is content-hashed and cached so repeated invocations skip regeneration.

pub mod engine;
pub mod templates;

pub use engine::{ForgeEngine, ForgeResult, ScriptLang, ToolSpec};
pub use templates::{TEMPLATES, Template, get_template, list_templates};
