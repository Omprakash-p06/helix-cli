use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub message: ChatMessage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerFlavor {
    LlamaCpp,
    KoboldCpp,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PermissionRequest {
    pub tool_name: String,
    pub arguments: Value,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum PermissionResponse {
    Allow,
    Deny,
}

#[async_trait::async_trait]
pub trait PermissionRequester: Send + Sync {
    async fn request_permission(&self, request: PermissionRequest) -> PermissionResponse;
}

/// Tracks the origin/trust level of content in the agent context.
/// The agent core uses this to prevent Untrusted content from being placed
/// in system-prompt or tool-schema positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provenance {
    /// Files and content from inside the configured workspace root
    Workspace,
    /// Built-in system prompts and tool definitions
    System,
    /// Content fetched by the research agent (cited, summarized)
    Research,
    /// Raw external content (web pages, user-supplied URLs, unverified files)
    Untrusted,
}

/// Associates content with its provenance for trust-aware context assembly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSource {
    pub content: String,
    pub provenance: Provenance,
}

/// Filters out content from Untrusted sources.
/// Used during context assembly to prevent untrusted data from reaching
/// system prompt or tool schema positions.
pub fn provenance_filter(sources: &[ContentSource]) -> Vec<ContentSource> {
    sources
        .iter()
        .filter(|s| s.provenance != Provenance::Untrusted)
        .cloned()
        .collect()
}

