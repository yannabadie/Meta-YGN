use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::schemars;
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use serde::Deserialize;

use crate::DaemonClient;

// ---------------------------------------------------------------------------
// Input parameter structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClassifyParams {
    /// The user prompt to classify
    pub prompt: String,
    /// Optional tool name for context
    pub tool_name: Option<String>,
    /// Optional tool input for context
    pub tool_input: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct VerifyParams {
    /// The tool that produced the output
    pub tool_name: String,
    /// The output returned by the tool
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

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub struct AletheiaHandler {
    daemon: DaemonClient,
    tool_router: ToolRouter<Self>,
}

impl AletheiaHandler {
    pub fn new(daemon: DaemonClient) -> Self {
        let tool_router = Self::tool_router();
        Self {
            daemon,
            tool_router,
        }
    }
}

// ---------------------------------------------------------------------------
// Tool definitions
// ---------------------------------------------------------------------------

#[tool_router]
impl AletheiaHandler {
    /// Classify a user prompt for metacognitive risk, intent, and tool-necessity.
    #[tool(name = "metacog_classify", description = "Classify a user prompt for metacognitive risk, intent, and tool-necessity.")]
    async fn metacog_classify(
        &self,
        Parameters(params): Parameters<ClassifyParams>,
    ) -> Result<String, String> {
        let body = serde_json::json!({
            "prompt": params.prompt,
            "tool_name": params.tool_name,
            "tool_input": params.tool_input,
        });
        self.daemon
            .post("/hooks/user-prompt-submit", &body)
            .await
            .map(|v| v.to_string())
            .map_err(|e| e.to_string())
    }

    /// Verify a tool's output against expectations and detect anomalies.
    #[tool(name = "metacog_verify", description = "Verify a tool's output against expectations and detect anomalies.")]
    async fn metacog_verify(
        &self,
        Parameters(params): Parameters<VerifyParams>,
    ) -> Result<String, String> {
        let body = serde_json::json!({
            "tool_name": params.tool_name,
            "tool_output": params.tool_output,
            "expected": params.expected,
        });
        self.daemon
            .post("/hooks/post-tool-use", &body)
            .await
            .map(|v| v.to_string())
            .map_err(|e| e.to_string())
    }

    /// Recall relevant memories (heuristics, episodic traces) by semantic query.
    #[tool(name = "metacog_recall", description = "Recall relevant memories (heuristics, episodic traces) by semantic query.")]
    async fn metacog_recall(
        &self,
        Parameters(params): Parameters<RecallParams>,
    ) -> Result<String, String> {
        let limit = params.limit.unwrap_or(10);
        let body = serde_json::json!({
            "query": params.query,
            "limit": limit,
        });
        self.daemon
            .post("/memory/recall", &body)
            .await
            .map(|v| v.to_string())
            .map_err(|e| e.to_string())
    }

    /// Get the current metacognitive status: health, fatigue, budget, and best heuristics.
    #[tool(name = "metacog_status", description = "Get the current metacognitive status: health, fatigue, budget, and best heuristics.")]
    async fn metacog_status(&self) -> Result<String, String> {
        let health = self.daemon.get("/health").await.unwrap_or_default();
        let fatigue = self
            .daemon
            .get("/profiler/fatigue")
            .await
            .unwrap_or_default();
        let budget = self.daemon.get("/budget").await.unwrap_or_default();
        let heuristics = self
            .daemon
            .get("/heuristics/best")
            .await
            .unwrap_or_default();

        let combined = serde_json::json!({
            "health": health,
            "fatigue": fatigue,
            "budget": budget,
            "heuristics": heuristics,
        });
        Ok(combined.to_string())
    }

    /// Prune a message array by sending it through the daemon's compaction proxy.
    #[tool(name = "metacog_prune", description = "Prune a message array by sending it through the daemon's compaction proxy.")]
    async fn metacog_prune(
        &self,
        Parameters(params): Parameters<PruneParams>,
    ) -> Result<String, String> {
        let messages: serde_json::Value =
            serde_json::from_str(&params.messages).map_err(|e| e.to_string())?;
        let body = serde_json::json!({ "messages": messages });
        self.daemon
            .post("/proxy/anthropic", &body)
            .await
            .map(|v| v.to_string())
            .map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// ServerHandler impl — delegates tool routing to the macro-generated router
// ---------------------------------------------------------------------------

#[tool_handler]
impl ServerHandler for AletheiaHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "aletheia-nexus".to_string(),
                title: Some("Aletheia Nexus MCP Bridge".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "Metacognitive runtime tools for coding-agent self-regulation.".to_string(),
            ),
        }
    }
}
