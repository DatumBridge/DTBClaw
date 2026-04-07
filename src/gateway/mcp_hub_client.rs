//! MCP WebSocket hub client — connects to datumbridge-mcp-ws-hub for scale (100+ devices).
//!
//! When MCP_HUB_URL, MCP_DEVICE_ID, and MCP_HUB_TOKEN env vars are set, DTBClaw connects
//! outbound to the hub. The token is server-generated (POST /api/v1/devices/register).
//! Enables DatumBridge Agent (AWS) to reach devices behind NAT without per-device tunnels.
//!
//! **Drift detection:** on connect, sends a `datumbridge` envelope with edge version + optional
//! git SHA (`DTB_GIT_SHA` at runtime or compile-time `DTB_GIT_SHA`). The hub answers with a
//! JSON-RPC notification `datumbridge/hub_handshake` (no `id`); we log it and continue.

use super::mcp_handler::{process_mcp_request, JsonRpcRequest};
use crate::gateway::AppState;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

fn edge_git_sha() -> String {
    if let Ok(s) = std::env::var("DTB_GIT_SHA") {
        let t = s.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    option_env!("DTB_GIT_SHA").unwrap_or("").to_string()
}

fn env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn mission_tool_name_from_env() -> String {
    std::env::var("MCP_EDGE_MISSION_TOOL_NAME")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "edge_agent_run".to_string())
}

fn edge_mission_protocol_from_env() -> i32 {
    std::env::var("EDGE_AGENT_RUN_MISSION_PROTOCOL")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .filter(|&p| p > 0)
        .unwrap_or(1)
}

fn edge_hello_envelope() -> String {
    let git_sha = edge_git_sha();
    let mut edge = serde_json::Map::new();
    edge.insert("protocol".into(), json!(1));
    edge.insert("version".into(), json!(env!("CARGO_PKG_VERSION")));
    edge.insert("git_sha".into(), json!(git_sha));
    edge.insert("server_name".into(), json!("dtbclaw"));

    if let Ok(v) = std::env::var("MCP_EDGE_LOCAL_LLM_AVAILABLE") {
        let t = v.trim().to_ascii_lowercase();
        if matches!(t.as_str(), "1" | "true" | "yes" | "on") {
            edge.insert("local_llm_available".into(), json!(true));
        } else if matches!(t.as_str(), "0" | "false" | "no" | "off") {
            edge.insert("local_llm_available".into(), json!(false));
        }
    }

    if env_truthy("MCP_EDGE_SUPPORTS_MISSION") {
        edge.insert("supports_edge_mission".into(), json!(true));
        edge.insert(
            "mission_tool_name".into(),
            json!(mission_tool_name_from_env()),
        );
        edge.insert(
            "edge_mission_protocol".into(),
            json!(edge_mission_protocol_from_env()),
        );
    }

    let hello = json!({
        "datumbridge": {
            "v": 1,
            "edge": serde_json::Value::Object(edge)
        }
    });
    hello.to_string()
}

/// Run the MCP hub client. Connects to hub, processes MCP messages, reconnects on disconnect.
pub async fn run_mcp_hub_client(
    state: Arc<AppState>,
    hub_url: String,
    device_id: String,
    token: String,
) {
    let ws_url = hub_url
        .replace("http://", "ws://")
        .replace("https://", "wss://");
    let url = format!(
        "{}/ws?device_id={}&token={}",
        ws_url.trim_end_matches('/'),
        urlencoding::encode(&device_id),
        urlencoding::encode(&token)
    );

    loop {
        match connect_and_serve(state.clone(), &url, &device_id).await {
            Ok(()) => tracing::info!("MCP hub client disconnected normally"),
            Err(e) => tracing::warn!("MCP hub client error: {e}; reconnecting in 5s"),
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn connect_and_serve(state: Arc<AppState>, url: &str, device_id: &str) -> Result<()> {
    let (mut ws_stream, _) = connect_async(url).await?;
    tracing::info!(target: "mcp_hub", device_id = %device_id, "MCP hub client connected");

    let hello = edge_hello_envelope();
    ws_stream
        .send(Message::Text(hello.into()))
        .await
        .map_err(|e| anyhow::anyhow!("send edge hello: {e}"))?;

    while let Some(msg) = ws_stream.next().await {
        let msg = msg.map_err(|e| anyhow::anyhow!("read: {e}"))?;
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        let body: JsonRpcRequest = match serde_json::from_str(&text) {
            Ok(b) => b,
            Err(_) => continue,
        };

        // Hub → edge: JSON-RPC notification (no id) for drift / handshake.
        if body.id.is_none() {
            if let Some(ref m) = body.method {
                if m == "datumbridge/hub_handshake" {
                    tracing::info!(
                        target: "mcp_hub",
                        device_id = %device_id,
                        params = ?body.params,
                        "hub handshake (drift / policy)"
                    );
                    continue;
                }
            }
        }

        let response = process_mcp_request(state.clone(), body).await;
        if let Err(e) = ws_stream
            .send(Message::Text(
                String::from_utf8_lossy(&response).into_owned().into(),
            ))
            .await
        {
            anyhow::bail!("send response: {e}");
        }
    }

    Ok(())
}
