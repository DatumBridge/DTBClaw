//! Shared MCP JSON-RPC handling for HTTP and WebSocket transports.
//! Used by mcp_server (POST /mcp) and mcp_hub_client (WebSocket to hub).

use crate::gateway::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: Option<String>,
    pub id: Option<Value>,
    pub method: Option<String>,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct McpToolDef {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

#[derive(Debug, Serialize)]
struct ToolsListResult {
    tools: Vec<McpToolDef>,
}

#[derive(Debug, Serialize)]
struct McpContentItem {
    #[serde(rename = "type")]
    typ: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct ToolsCallResult {
    content: Vec<McpContentItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_error: Option<bool>,
}

#[derive(Debug, Serialize)]
struct JsonRpcSuccess {
    jsonrpc: String,
    id: Value,
    result: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    jsonrpc: String,
    id: Value,
    error: JsonRpcErrorObj,
}

#[derive(Debug, Serialize)]
struct JsonRpcErrorObj {
    code: i32,
    message: String,
}

/// Process an MCP JSON-RPC request and return the response as JSON bytes.
pub async fn process_mcp_request(state: Arc<AppState>, body: JsonRpcRequest) -> Vec<u8> {
    let id = body.id.unwrap_or(Value::Null);
    let method = match &body.method {
        Some(m) => m.as_str(),
        None => {
            let err = JsonRpcError {
                jsonrpc: "2.0".into(),
                id,
                error: JsonRpcErrorObj {
                    code: -32600,
                    message: "Invalid Request: method required".into(),
                },
            };
            return serde_json::to_vec(&err).unwrap_or_default();
        }
    };

    let result: Value = match method {
        "initialize" => process_initialize(id),
        "tools/list" => process_tools_list(state, id),
        "tools/call" => process_tools_call(state, id, body.params).await,
        _ => serde_json::to_value(JsonRpcError {
            jsonrpc: "2.0".into(),
            id,
            error: JsonRpcErrorObj {
                code: -32601,
                message: format!("Method not found: {method}"),
            },
        })
        .unwrap_or(Value::Null),
    };

    serde_json::to_vec(&result).unwrap_or_default()
}

fn process_initialize(id: Value) -> Value {
    let result = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "serverInfo": {
            "name": "dtbclaw",
            "version": env!("CARGO_PKG_VERSION")
        }
    });
    serde_json::to_value(JsonRpcSuccess {
        jsonrpc: "2.0".into(),
        id,
        result,
    })
    .unwrap_or(Value::Null)
}

fn process_tools_list(state: Arc<AppState>, id: Value) -> Value {
    let tools: Vec<McpToolDef> = state
        .tools_registry_exec
        .iter()
        .map(|t| {
            let spec = t.spec();
            McpToolDef {
                name: spec.name,
                description: spec.description,
                input_schema: spec.parameters,
            }
        })
        .collect();

    let result = ToolsListResult { tools };
    serde_json::to_value(JsonRpcSuccess {
        jsonrpc: "2.0".into(),
        id,
        result: serde_json::to_value(result).unwrap_or(Value::Null),
    })
    .unwrap_or(Value::Null)
}

async fn process_tools_call(state: Arc<AppState>, id: Value, params: Option<Value>) -> Value {
    let params = match params {
        Some(p) => p,
        None => {
            return serde_json::to_value(JsonRpcError {
                jsonrpc: "2.0".into(),
                id,
                error: JsonRpcErrorObj {
                    code: -32602,
                    message: "Invalid params: params required for tools/call".into(),
                },
            })
            .unwrap_or(Value::Null);
        }
    };

    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let arguments = params
        .get("arguments")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<String, Value>>()
        })
        .unwrap_or_default();

    if name.is_empty() {
        return serde_json::to_value(JsonRpcError {
            jsonrpc: "2.0".into(),
            id,
            error: JsonRpcErrorObj {
                code: -32602,
                message: "Invalid params: name required".into(),
            },
        })
        .unwrap_or(Value::Null);
    }

    let tool = state.tools_registry_exec.iter().find(|t| t.name() == name);

    let Some(tool) = tool else {
        return serde_json::to_value(JsonRpcError {
            jsonrpc: "2.0".into(),
            id,
            error: JsonRpcErrorObj {
                code: -32602,
                message: format!("Tool not found: {name}"),
            },
        })
        .unwrap_or(Value::Null);
    };

    let args = serde_json::Value::Object(
        arguments
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect::<serde_json::Map<_, _>>(),
    );

    match tool.execute(args).await {
        Ok(result) => {
            let text = if result.success {
                result.output
            } else {
                result
                    .error
                    .unwrap_or_else(|| "Tool execution failed".into())
            };
            let call_result = ToolsCallResult {
                content: vec![McpContentItem {
                    typ: "text".into(),
                    text,
                }],
                is_error: Some(!result.success),
            };
            serde_json::to_value(JsonRpcSuccess {
                jsonrpc: "2.0".into(),
                id,
                result: serde_json::to_value(call_result).unwrap_or(Value::Null),
            })
            .unwrap_or(Value::Null)
        }
        Err(e) => serde_json::to_value(JsonRpcError {
            jsonrpc: "2.0".into(),
            id,
            error: JsonRpcErrorObj {
                code: -32603,
                message: e.to_string(),
            },
        })
        .unwrap_or(Value::Null),
    }
}
