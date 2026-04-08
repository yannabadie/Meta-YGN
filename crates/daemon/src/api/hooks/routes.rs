use axum::routing::post;
use axum::Router;

use crate::app_state::AppState;

use super::post_tool_use::post_tool_use;
use super::pre_tool_use::pre_tool_use;
use super::session_end::{analyze, session_end};
use super::stop::stop;
use super::user_prompt_submit::user_prompt_submit;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/hooks/pre-tool-use", post(pre_tool_use))
        .route("/hooks/post-tool-use", post(post_tool_use))
        .route("/hooks/user-prompt-submit", post(user_prompt_submit))
        .route("/hooks/stop", post(stop))
        .route("/hooks/session-end", post(session_end))
        .route("/hooks/analyze", post(analyze))
}
