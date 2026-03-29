//! Prints JSON catalog for DatumBridge (ws-hub + MCP registry).
//! Run from DTBClaw repo: `cargo run --bin export_dtbclaw_edge_catalog --quiet > dtbclaw_edge_catalog.json`

use std::io::{self, Write};

use anyhow::Result;

fn main() -> Result<()> {
    let v = octoclaw::tools::datumbridge_catalog::build_datumbridge_edge_manifest()?;
    let s = serde_json::to_string_pretty(&v)?;
    io::stdout().write_all(s.as_bytes())?;
    io::stdout().write_all(b"\n")?;
    Ok(())
}
