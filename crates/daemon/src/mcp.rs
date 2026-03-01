//! MCP stdio server handler — fuses the 5 MCP tools directly into the daemon.
//!
//! Instead of routing through HTTP (as `mcp-bridge` does), these handlers
//! access [`AppState`] directly, removing a network hop and the requirement
//! to have the HTTP daemon running separately.
//!
//! Gated behind `#[cfg(feature = "mcp")]`.

#[cfg(feature = "mcp")]
pub mod mcp_handler {
    use rmcp::handler::server::router::tool::ToolRouter;
    use rmcp::handler::server::wrapper::Parameters;
    use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
    use rmcp::schemars;
    use rmcp::{ServerHandler, tool, tool_handler, tool_router};
    use serde::Deserialize;

    use crate::app_state::AppState;

    // -----------------------------------------------------------------------
    // Input parameter structs (mirrored from mcp-bridge/handler.rs)
    // -----------------------------------------------------------------------

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct ClassifyParams {
        /// The user prompt to classify
        pub prompt: String,
        /// Optional tool name for context
        pub tool_name: Option<String>,
        /// Optional tool input JSON for context.
        /// Note: MCP callers may omit this; the classify pipeline still works
        /// without it (tool_input is only used for deeper risk analysis).
        pub tool_input: Option<String>,
    }

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct VerifyParams {
        /// The tool that produced the output
        pub tool_name: String,
        /// The tool output to verify (maps to `HookInput.tool_response`)
        pub tool_output: String,
        /// Optional expected output for comparison
        pub expected: Option<String>,
    }

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct RecallParams {
        /// The query to search memory for
        pub query: String,
        /// Maximum number of results to return (default: 10)
        pub limit: Option<u32>,
    }

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct PruneParams {
        /// A JSON array of messages to prune
        pub messages: String,
    }

    // -----------------------------------------------------------------------
    // Handler
    // -----------------------------------------------------------------------

    pub struct AletheiaHandler {
        state: AppState,
        tool_router: ToolRouter<Self>,
    }

    impl AletheiaHandler {
        pub fn new(state: AppState) -> Self {
            let tool_router = Self::tool_router();
            Self { state, tool_router }
        }
    }

    // -----------------------------------------------------------------------
    // Tool definitions — direct AppState access (no HTTP round-trip)
    // -----------------------------------------------------------------------

    #[tool_router]
    impl AletheiaHandler {
        /// Classify a user prompt for metacognitive risk, intent, and tool-necessity.
        ///
        /// Constructs a `HookInput` with `hook_event_name = UserPromptSubmit`.
        /// `tool_input` is optionally parsed from the caller's JSON string; MCP
        /// callers typically omit it since they don't have structured tool input.
        #[tool(
            name = "metacog_classify",
            description = "Classify a user prompt for metacognitive risk, intent, and tool-necessity."
        )]
        async fn metacog_classify(
            &self,
            Parameters(params): Parameters<ClassifyParams>,
        ) -> Result<String, String> {
            let input = metaygn_shared::protocol::HookInput {
                hook_event_name: metaygn_shared::protocol::HookEvent::UserPromptSubmit,
                prompt: Some(params.prompt),
                tool_name: params.tool_name,
                tool_input: params
                    .tool_input
                    .and_then(|s| serde_json::from_str(&s).ok()),
                session_id: None,
                cwd: None,
                tool_response: None,
                error: None,
                last_assistant_message: None,
                source: None,
                reason: None,
                trigger: None,
            };
            let mut ctx = metaygn_core::context::LoopContext::new(input);
            self.state.control_loop.run_range(&mut ctx, 0, 6);
            Ok(serde_json::to_string_pretty(&serde_json::json!({
                "task_type": ctx.task_type.map(|t| format!("{:?}", t)),
                "risk": format!("{:?}", ctx.risk),
                "strategy": format!("{:?}", ctx.strategy),
                "difficulty": ctx.difficulty,
                "competence": ctx.competence,
                "tool_necessary": ctx.tool_necessary,
                "metacog": ctx.metacog_vector.compact_encode(),
            }))
            .unwrap_or_default())
        }

        /// Verify a tool's output against expectations and detect anomalies.
        #[tool(
            name = "metacog_verify",
            description = "Verify a tool's output against expectations and detect anomalies."
        )]
        async fn metacog_verify(
            &self,
            Parameters(params): Parameters<VerifyParams>,
        ) -> Result<String, String> {
            let input = metaygn_shared::protocol::HookInput {
                hook_event_name: metaygn_shared::protocol::HookEvent::PostToolUse,
                tool_name: Some(params.tool_name),
                tool_response: Some(params.tool_output),
                session_id: None,
                cwd: None,
                tool_input: None,
                prompt: None,
                error: None,
                last_assistant_message: None,
                source: None,
                reason: None,
                trigger: None,
            };
            let mut ctx = metaygn_core::context::LoopContext::new(input);
            // Run verify + calibrate stages (stages 8 and 9, 0-indexed as 7..10)
            self.state.control_loop.run_range(&mut ctx, 7, 10);
            Ok(serde_json::to_string_pretty(&serde_json::json!({
                "verification_results": ctx.verification_results,
                "metacog": ctx.metacog_vector.compact_encode(),
            }))
            .unwrap_or_default())
        }

        /// Recall relevant memories (heuristics, episodic traces) by semantic query.
        #[tool(
            name = "metacog_recall",
            description = "Recall relevant memories (heuristics, episodic traces) by semantic query."
        )]
        async fn metacog_recall(
            &self,
            Parameters(params): Parameters<RecallParams>,
        ) -> Result<String, String> {
            let limit = params.limit.unwrap_or(10);
            match self.state.memory.search_events(&params.query, limit).await {
                Ok(events) => {
                    let results: Vec<serde_json::Value> = events
                        .iter()
                        .map(|e| {
                            serde_json::json!({
                                "event_type": e.event_type,
                                "payload": e.payload,
                                "timestamp": e.timestamp,
                            })
                        })
                        .collect();
                    Ok(
                        serde_json::to_string_pretty(&serde_json::json!({"events": results}))
                            .unwrap_or_default(),
                    )
                }
                Err(e) => Err(format!("recall failed: {e}")),
            }
        }

        /// Get the current metacognitive status: health, fatigue, budget, and best heuristics.
        #[tool(
            name = "metacog_status",
            description = "Get the current metacognitive status: health, fatigue, budget, and best heuristics."
        )]
        async fn metacog_status(&self) -> Result<String, String> {
            let event_count = self.state.memory.event_count().await.unwrap_or(0);
            let node_count = self.state.graph.node_count().await.unwrap_or(0);

            let fatigue_report = {
                let profiler = self.state.fatigue.lock().expect("fatigue mutex poisoned");
                profiler.assess()
            };

            let budget_json = {
                let budget = self.state.budget.lock().expect("budget mutex poisoned");
                serde_json::to_value(&*budget).unwrap_or_default()
            };

            let heuristics_json = {
                let evolver = self.state.evolver.lock().expect("evolver mutex poisoned");
                match evolver.best() {
                    Some(best) => serde_json::to_value(best).unwrap_or_default(),
                    None => serde_json::json!(null),
                }
            };

            Ok(serde_json::to_string_pretty(&serde_json::json!({
                "status": "ok",
                "events": event_count,
                "graph_nodes": node_count,
                "fatigue": {
                    "score": fatigue_report.score,
                    "high_friction": fatigue_report.high_friction,
                    "signals": fatigue_report.signals,
                    "recommendation": fatigue_report.recommendation,
                },
                "budget": budget_json,
                "heuristics": heuristics_json,
            }))
            .unwrap_or_default())
        }

        /// Prune a message array by detecting error loops and suggesting compaction.
        ///
        /// Parses the input as `Vec<pruner::Message>`, runs `ContextPruner::analyze`,
        /// and returns the analysis (consecutive errors, suggested injection, count).
        #[tool(
            name = "metacog_prune",
            description = "Prune a message array by sending it through the daemon's compaction proxy."
        )]
        async fn metacog_prune(
            &self,
            Parameters(params): Parameters<PruneParams>,
        ) -> Result<String, String> {
            use crate::proxy::pruner::{ContextPruner, Message};

            let messages: Vec<Message> =
                serde_json::from_str(&params.messages).unwrap_or_default();
            let pruner = ContextPruner::with_defaults();
            let analysis = pruner.analyze(&messages);
            Ok(serde_json::to_string_pretty(&serde_json::json!({
                "consecutive_errors": analysis.consecutive_errors,
                "suggested_injection": analysis.suggested_injection,
                "messages_analyzed": messages.len(),
            }))
            .unwrap_or_default())
        }
    }

    // -----------------------------------------------------------------------
    // ServerHandler impl — delegates tool routing to the macro-generated router
    // -----------------------------------------------------------------------

    #[tool_handler]
    impl ServerHandler for AletheiaHandler {
        fn get_info(&self) -> ServerInfo {
            ServerInfo {
                protocol_version: Default::default(),
                capabilities: ServerCapabilities::builder().enable_tools().build(),
                server_info: Implementation {
                    name: "aletheia-nexus".to_string(),
                    title: Some("Aletheia Nexus MCP (fused)".to_string()),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    ..Default::default()
                },
                instructions: Some(
                    "Metacognitive runtime tools for coding-agent self-regulation. \
                     This server runs inside the daemon process — no HTTP hop required."
                        .to_string(),
                ),
            }
        }
    }
}
