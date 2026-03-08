//! MCP (Model Context Protocol) server endpoint for DatumBridge Tool Registry integration.
//!
//! Exposes DTBClaw tools via POST /mcp with JSON-RPC: initialize, tools/list, tools/call.
//! Compatible with datumbridge-mcp client (FetchMCPTools, ExecuteTool).

use super::mcp_handler::{process_mcp_request, JsonRpcRequest};
use crate::gateway::AppState;
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// Handle POST /mcp — MCP JSON-RPC endpoint
pub async fn handle_mcp(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(body): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let state = Arc::new(state);
    let response_bytes = process_mcp_request(state, body.clone()).await;

    let mut headers = HeaderMap::new();
    if body.method.as_deref() == Some("initialize") {
        let session_id = Uuid::new_v4().to_string();
        headers.insert(
            header::HeaderName::from_static("mcp-session-id"),
            HeaderValue::from_str(&session_id).expect("UUID is valid HeaderValue"),
        );
    }

    (StatusCode::OK, headers, response_bytes).into_response()
}
