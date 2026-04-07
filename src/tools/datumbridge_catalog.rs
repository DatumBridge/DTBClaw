//! JSON catalog of native tools for DatumBridge MCP WS Hub and datumbridge-mcp registry.

use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use tempfile::TempDir;

use crate::config::{BrowserConfig, Config, HttpRequestConfig, MemoryConfig, WebFetchConfig};
use crate::memory::create_memory;
use crate::security::SecurityPolicy;
use crate::tools::all_tools;

#[derive(Serialize)]
struct EdgeManifest {
    profile: String,
    description: String,
    tools: Vec<EdgeTool>,
}

#[derive(Serialize)]
struct EdgeTool {
    #[serde(rename = "registryId")]
    registry_id: String,
    name: String,
    #[serde(rename = "mcpToolName")]
    mcp_tool_name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
}

/// Build manifest JSON for edge devices (typical gateway: browser + HTTP + web_fetch enabled).
pub fn build_datumbridge_edge_manifest() -> anyhow::Result<serde_json::Value> {
    let tmp = TempDir::new()?;
    let security = Arc::new(SecurityPolicy::default());
    let mem_cfg = MemoryConfig {
        backend: "markdown".into(),
        ..MemoryConfig::default()
    };
    let mem: Arc<dyn crate::memory::Memory> = Arc::from(create_memory(&mem_cfg, tmp.path(), None)?);

    let mut browser = BrowserConfig::default();
    browser.enabled = true;
    browser.allowed_domains = vec!["*".into()];

    let mut http = HttpRequestConfig::default();
    http.enabled = true;
    http.allowed_domains = vec!["*".into()];

    let mut web_fetch = WebFetchConfig::default();
    web_fetch.enabled = true;

    let cfg = Config {
        workspace_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("config.toml"),
        ..Config::default()
    };
    let cfg_arc = Arc::new(cfg.clone());

    let tools = all_tools(
        cfg_arc,
        &security,
        mem,
        None,
        None,
        &browser,
        &http,
        &web_fetch,
        tmp.path(),
        &HashMap::new(),
        None,
        &cfg,
    );

    let mut out_tools: Vec<EdgeTool> = tools
        .iter()
        .map(|t| {
            let mut metadata = None;
            if t.name() == "edge_agent_run" {
                metadata = Some(serde_json::json!({
                    "edgeDevice": true,
                    "requiresEdgeMission": true
                }));
            }
            EdgeTool {
                registry_id: format!("edge_dtbclaw_{}", t.name().replace(['.', '-'], "_")),
                name: format!("Edge: {}", t.name()),
                mcp_tool_name: t.name().to_string(),
                description: t.description().to_string(),
                input_schema: t.parameters_schema(),
                metadata,
            }
        })
        .collect();

    out_tools.sort_by(|a, b| a.mcp_tool_name.cmp(&b.mcp_tool_name));

    let manifest = EdgeManifest {
        profile: "dtbclaw-default".into(),
        description: "Native octoclaw / DTBClaw tools for edge devices via MCP WS Hub. Regenerate with: cargo run --bin export_dtbclaw_edge_catalog".into(),
        tools: out_tools,
    };

    Ok(serde_json::to_value(&manifest)?)
}
