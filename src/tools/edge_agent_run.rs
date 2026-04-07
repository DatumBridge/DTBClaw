//! Bounded edge mission tool (Approach 2 / 3): validates master objectives against operator budgets.
//!
//! Registered only when `MCP_EDGE_SUPPORTS_MISSION` is `1`/`true`/`on`. Gateway budgets are enforced
//! here as hard caps; see env vars `EDGE_AGENT_RUN_MAX_STEPS`, `EDGE_AGENT_RUN_MAX_WALL_SEC`,
//! `EDGE_AGENT_RUN_MAX_OBJECTIVE_CHARS`.

use async_trait::async_trait;
use serde_json::json;

use super::traits::{Tool, ToolResult};

const DEFAULT_TOOL_NAME: &str = "edge_agent_run";

/// Tool exposing a bounded mission contract for DatumBridge master ↔ edge (see docs/ADK_APPROACH2).
pub struct EdgeAgentRunTool {
    name: String,
    max_steps_cap: u32,
    max_wall_sec_cap: u64,
    max_objective_chars: usize,
    mission_protocol: i32,
}

impl EdgeAgentRunTool {
    /// Whether the hub/catalog should advertise and relay this tool (matches edge hello `supports_edge_mission`).
    pub fn enabled_from_env() -> bool {
        matches_env_truthy("MCP_EDGE_SUPPORTS_MISSION")
    }

    pub fn from_env() -> Self {
        let name = std::env::var("MCP_EDGE_MISSION_TOOL_NAME")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_TOOL_NAME.to_string());
        let max_steps_cap = parse_u32_env("EDGE_AGENT_RUN_MAX_STEPS", 16).clamp(1, 256);
        let max_wall_sec_cap = parse_u64_env("EDGE_AGENT_RUN_MAX_WALL_SEC", 600).clamp(1, 86_400);
        let max_objective_chars =
            parse_usize_env("EDGE_AGENT_RUN_MAX_OBJECTIVE_CHARS", 16_384).clamp(256, 512_000);
        let mission_protocol = parse_i32_env("EDGE_AGENT_RUN_MISSION_PROTOCOL", 1).max(1);
        Self {
            name,
            max_steps_cap,
            max_wall_sec_cap,
            max_objective_chars,
            mission_protocol,
        }
    }

    fn clamp_u32(v: Option<u32>, cap: u32) -> u32 {
        v.unwrap_or(cap).min(cap).max(1)
    }

    fn clamp_u64(v: Option<u64>, cap: u64) -> u64 {
        v.unwrap_or(cap).min(cap).max(1)
    }
}

fn matches_env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn parse_u32_env(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(default)
}

fn parse_u64_env(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(default)
}

fn parse_usize_env(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(default)
}

fn parse_i32_env(key: &str, default: i32) -> i32 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(default)
}

#[async_trait]
impl Tool for EdgeAgentRunTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Bounded edge mission: run a high-level objective on this gateway with strict max steps and wall time. Enforces operator caps; returns an acceptance envelope with negotiated budgets (Approach 2/3 contract)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "objective": {
                    "type": "string",
                    "description": "High-level task for the edge agent loop."
                },
                "max_steps": {
                    "type": "integer",
                    "description": "Requested maximum tool/agent steps (clamped by gateway)."
                },
                "max_wall_seconds": {
                    "type": "integer",
                    "description": "Requested wall-clock budget in seconds (clamped by gateway)."
                },
                "mission_protocol": {
                    "type": "integer",
                    "description": "Expected mission protocol version from master (must match edge)."
                },
                "correlation_id": {
                    "type": "string",
                    "description": "Optional id to join master run ↔ edge mission in logs."
                },
                "tool_allowlist": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional subset of MCP tool names the mission may invoke (gateway may further restrict)."
                }
            },
            "required": ["objective"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let objective = args
            .get("objective")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if objective.is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("objective is required".into()),
            });
        }
        if objective.chars().count() > self.max_objective_chars {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "objective exceeds max length ({}, max {})",
                    objective.chars().count(),
                    self.max_objective_chars
                )),
            });
        }

        let req_proto = args
            .get("mission_protocol")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        if req_proto > 0 && req_proto != self.mission_protocol {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "mission_protocol mismatch: edge supports {}, master requested {}",
                    self.mission_protocol, req_proto
                )),
            });
        }

        let max_steps = Self::clamp_u32(
            args.get("max_steps")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            self.max_steps_cap,
        );
        let max_wall = Self::clamp_u64(
            args.get("max_wall_seconds").and_then(|v| v.as_u64()),
            self.max_wall_sec_cap,
        );

        // Tool allowlist: optional validation against env GATEWAY_MISSION_TOOL_ALLOWLIST (comma-separated).
        if let Ok(raw) = std::env::var("GATEWAY_MISSION_TOOL_ALLOWLIST") {
            let allow: std::collections::HashSet<String> = raw
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !allow.is_empty() {
                if let Some(arr) = args.get("tool_allowlist").and_then(|v| v.as_array()) {
                    for item in arr {
                        let Some(name) = item.as_str() else { continue };
                        if !allow.contains(name.trim()) {
                            return Ok(ToolResult {
                                success: false,
                                output: String::new(),
                                error: Some(format!(
                                    "tool_allowlist contains disallowed tool {:?} (not in GATEWAY_MISSION_TOOL_ALLOWLIST)",
                                    name
                                )),
                            });
                        }
                    }
                }
            }
        }

        let correlation_id = args
            .get("correlation_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let out = json!({
            "ok": true,
            "edge_mission_protocol": self.mission_protocol,
            "mission_tool": self.name,
            "accepted_budget": {
                "max_steps": max_steps,
                "max_wall_seconds": max_wall,
            },
            "correlation_id": correlation_id,
            "note": "Budgets validated. Wire local autonomous execution to this envelope in product builds; until then masters may use this as a contract probe."
        });
        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&out)?,
            error: None,
        })
    }
}
