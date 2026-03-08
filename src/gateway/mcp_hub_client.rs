//! MCP WebSocket hub client — connects to datumbridge-mcp-ws-hub for scale (100+ devices).
//!
//! When MCP_HUB_URL, MCP_DEVICE_ID, and MCP_HUB_TOKEN env vars are set, DTBClaw connects
//! outbound to the hub. The token is server-generated (POST /api/v1/devices/register).
//! Enables DatumBridge Agent (AWS) to reach devices behind NAT without per-device tunnels.

use super::mcp_handler::{process_mcp_request, JsonRpcRequest};
use crate::gateway::AppState;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

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
    tracing::info!("MCP hub client connected: device_id={device_id}");

    // Optional: send register message (hub also accepts device_id from query param)
    let register = serde_json::json!({
        "type": "register",
        "device_id": device_id
    });
    if let Ok(msg) = serde_json::to_string(&register) {
        ws_stream
            .send(Message::Text(msg.into()))
            .await
            .map_err(|e| anyhow::anyhow!("send register: {e}"))?;
    }

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
