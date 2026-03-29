//! Cursor IDE CLI tool — open files, diff, manage extensions, send prompts,
//! and control Cursor editor instances from the agent loop.
//!
//! Wraps the `cursor` CLI binary (ships with Cursor IDE and must be on PATH).

use super::traits::{Tool, ToolResult};
use crate::security::{AutonomyLevel, SecurityPolicy};
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;
use std::sync::Arc;

const PROMPT_DIR: &str = ".cursor";
const PROMPT_FILE: &str = ".cursor/agent-prompt.md";
const AGENT_TIMEOUT_SECS: u64 = 300;
const AGENT_OUTPUT_LOG: &str = ".cursor/agent-output.log";
const AGENT_STATUS_FILE: &str = ".cursor/agent-status.json";

pub struct CursorCliTool {
    security: Arc<SecurityPolicy>,
    workspace_dir: std::path::PathBuf,
}

impl CursorCliTool {
    pub fn new(security: Arc<SecurityPolicy>, workspace_dir: std::path::PathBuf) -> Self {
        Self {
            security,
            workspace_dir,
        }
    }

    fn requires_write_access(operation: &str) -> bool {
        matches!(
            operation,
            "install_extension" | "uninstall_extension" | "new_window" | "prompt" | "agent"
        )
    }

    /// Resolve the `cursor` binary name. The CLI ships as `cursor` on all
    /// platforms when the user runs "Install 'cursor' command in PATH" from
    /// inside the Cursor IDE.
    fn cursor_bin() -> &'static str {
        "cursor"
    }

    fn sanitize_path(raw: &str) -> anyhow::Result<String> {
        if raw.contains("$(")
            || raw.contains('`')
            || raw.contains('|')
            || raw.contains(';')
            || raw.contains('>')
            || raw.contains('<')
            || raw.contains('&')
        {
            anyhow::bail!("Path contains disallowed shell metacharacters: {raw}");
        }
        Ok(raw.to_string())
    }

    fn sanitize_extension_id(id: &str) -> anyhow::Result<String> {
        let trimmed = id.trim();
        if trimmed.is_empty() {
            anyhow::bail!("Extension ID cannot be empty");
        }
        if !trimmed
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            anyhow::bail!(
                "Extension ID contains invalid characters (only alphanumeric, '.', '-', '_' allowed): {trimmed}"
            );
        }
        Ok(trimmed.to_string())
    }

    async fn run_cursor_command(&self, args: &[&str]) -> anyhow::Result<String> {
        let output = tokio::process::Command::new(Self::cursor_bin())
            .args(args)
            .current_dir(&self.workspace_dir)
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow::anyhow!(
                        "cursor CLI not found on PATH. Open Cursor IDE and run \
                         'Shell Command: Install cursor command in PATH' from the command palette."
                    )
                } else {
                    anyhow::anyhow!("Failed to run cursor CLI: {e}")
                }
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            let detail = if stderr.trim().is_empty() {
                stdout.trim().to_string()
            } else {
                stderr.trim().to_string()
            };
            anyhow::bail!("cursor CLI exited with {}: {detail}", output.status);
        }

        Ok(if stdout.trim().is_empty() {
            stderr
        } else {
            stdout
        })
    }

    async fn cmd_open(&self, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let path = Self::sanitize_path(path)?;

        let reuse_window = args
            .get("reuse_window")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let wait = args.get("wait").and_then(|v| v.as_bool()).unwrap_or(false);

        let mut cli_args = Vec::new();
        if reuse_window {
            cli_args.push("--reuse-window");
        } else {
            cli_args.push("--new-window");
        }
        if wait {
            cli_args.push("--wait");
        }
        cli_args.push(&path);

        self.run_cursor_command(&cli_args).await?;

        let abs_display = if Path::new(&path).is_absolute() {
            path.clone()
        } else {
            self.workspace_dir.join(&path).display().to_string()
        };

        Ok(ToolResult {
            success: true,
            output: format!("Opened in Cursor: {abs_display}"),
            error: None,
        })
    }

    async fn cmd_goto(&self, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
        let file = args
            .get("file")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'file' parameter for goto"))?;
        let file = Self::sanitize_path(file)?;

        let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(1);
        let column = args.get("column").and_then(|v| v.as_u64()).unwrap_or(1);

        let goto_spec = format!("{file}:{line}:{column}");

        self.run_cursor_command(&["--goto", &goto_spec]).await?;

        Ok(ToolResult {
            success: true,
            output: format!("Opened {file} at line {line}, column {column}"),
            error: None,
        })
    }

    async fn cmd_diff(&self, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
        let file1 = args
            .get("file1")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'file1' parameter for diff"))?;
        let file2 = args
            .get("file2")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'file2' parameter for diff"))?;

        let file1 = Self::sanitize_path(file1)?;
        let file2 = Self::sanitize_path(file2)?;

        self.run_cursor_command(&["--diff", &file1, &file2]).await?;

        Ok(ToolResult {
            success: true,
            output: format!("Opened diff view: {file1} ↔ {file2}"),
            error: None,
        })
    }

    async fn cmd_install_extension(&self, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
        let ext_id = args
            .get("extension_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'extension_id' parameter"))?;
        let ext_id = Self::sanitize_extension_id(ext_id)?;

        let output = self
            .run_cursor_command(&["--install-extension", &ext_id])
            .await?;

        Ok(ToolResult {
            success: true,
            output: format!("Installed extension {ext_id}: {}", output.trim()),
            error: None,
        })
    }

    async fn cmd_uninstall_extension(
        &self,
        args: &serde_json::Value,
    ) -> anyhow::Result<ToolResult> {
        let ext_id = args
            .get("extension_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'extension_id' parameter"))?;
        let ext_id = Self::sanitize_extension_id(ext_id)?;

        let output = self
            .run_cursor_command(&["--uninstall-extension", &ext_id])
            .await?;

        Ok(ToolResult {
            success: true,
            output: format!("Uninstalled extension {ext_id}: {}", output.trim()),
            error: None,
        })
    }

    async fn cmd_list_extensions(&self) -> anyhow::Result<ToolResult> {
        let output = self.run_cursor_command(&["--list-extensions"]).await?;

        let extensions: Vec<&str> = output
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&json!({
                "count": extensions.len(),
                "extensions": extensions,
            }))
            .unwrap_or_default(),
            error: None,
        })
    }

    async fn cmd_new_window(&self, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
        let path = args.get("path").and_then(|v| v.as_str());

        let mut cli_args = vec!["--new-window"];
        let sanitized;
        if let Some(p) = path {
            sanitized = Self::sanitize_path(p)?;
            cli_args.push(&sanitized);
        }

        self.run_cursor_command(&cli_args).await?;

        let msg = match path {
            Some(p) => format!("Opened new Cursor window at {p}"),
            None => "Opened new Cursor window".to_string(),
        };

        Ok(ToolResult {
            success: true,
            output: msg,
            error: None,
        })
    }

    async fn cmd_status(&self) -> anyhow::Result<ToolResult> {
        let output = self.run_cursor_command(&["--version"]).await?;

        let version = output
            .lines()
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string();

        let extensions = self
            .run_cursor_command(&["--list-extensions"])
            .await
            .ok()
            .map(|out| {
                out.lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&json!({
                "version": version,
                "extension_count": extensions.len(),
                "extensions": extensions,
                "workspace": self.workspace_dir.display().to_string(),
            }))
            .unwrap_or_default(),
            error: None,
        })
    }

    async fn cmd_prompt(&self, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'message' parameter for prompt"))?;

        if message.trim().is_empty() {
            anyhow::bail!("Prompt message cannot be empty");
        }

        let file_context = args.get("file").and_then(|v| v.as_str());
        let mode = args
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("composer");

        let prompt_dir = self.workspace_dir.join(PROMPT_DIR);
        if !prompt_dir.exists() {
            tokio::fs::create_dir_all(&prompt_dir).await?;
        }

        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let mut content = format!("# Agent Prompt — {timestamp}\n\n");

        if let Some(ctx_file) = file_context {
            content.push_str(&format!("**Target file:** `{ctx_file}`\n\n"));
        }
        content.push_str(&format!("**Mode:** {mode}\n\n"));
        content.push_str("---\n\n");
        content.push_str(message);
        content.push('\n');

        if mode == "rules" {
            content.push_str("\n---\n\n");
            content.push_str(
                "_This prompt was written as a project rule. \
                 Copy the content above into `.cursor/rules/*.mdc` to make it permanent._\n",
            );
        }

        let prompt_path = self.workspace_dir.join(PROMPT_FILE);
        tokio::fs::write(&prompt_path, &content).await?;

        let mut cli_args = vec!["--reuse-window"];
        if let Some(ctx_file) = file_context {
            let sanitized = Self::sanitize_path(ctx_file)?;
            self.run_cursor_command(&["--reuse-window", &sanitized])
                .await
                .ok();
        }

        let prompt_path_str = prompt_path.display().to_string();
        cli_args.push(&prompt_path_str);
        self.run_cursor_command(&cli_args).await?;

        let guidance = match mode {
            "composer" => {
                "Prompt written and opened in Cursor. \
                 Open Composer (Cmd+I / Ctrl+I), paste or reference the prompt, and run."
            }
            "chat" => {
                "Prompt written and opened in Cursor. \
                 Open Chat (Cmd+L / Ctrl+L), paste or reference the prompt, and ask."
            }
            "edit" => {
                "Prompt written and opened in Cursor. \
                 Select code in the target file, press Cmd+K / Ctrl+K, and paste the prompt."
            }
            "rules" => {
                "Prompt written and opened in Cursor. \
                 Copy into .cursor/rules/*.mdc to save as a permanent project rule."
            }
            _ => "Prompt written and opened in Cursor.",
        };

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&json!({
                "prompt_file": prompt_path_str,
                "mode": mode,
                "message_length": message.len(),
                "guidance": guidance,
            }))
            .unwrap_or_default(),
            error: None,
        })
    }

    async fn cmd_agent(&self, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'message' parameter for agent"))?;

        if message.trim().is_empty() {
            anyhow::bail!("Agent prompt cannot be empty");
        }

        let model = args.get("model").and_then(|v| v.as_str());
        let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(true);
        let headless = args
            .get("headless")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let workspace = args
            .get("workspace")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.workspace_dir.display().to_string());
        let workspace = Self::sanitize_path(&workspace)?;

        if headless {
            return self
                .run_agent_headless(message, &workspace, model, force, args)
                .await;
        }

        self.open_agent_in_terminal(message, &workspace, model, force)
            .await
    }

    /// Opens `cursor agent` in a visible terminal window. Returns immediately.
    /// Output is tee'd to `.cursor/agent-output.log` and status is tracked
    /// in `.cursor/agent-status.json` — poll with `agent_status` operation.
    async fn open_agent_in_terminal(
        &self,
        message: &str,
        workspace: &str,
        model: Option<&str>,
        force: bool,
    ) -> anyhow::Result<ToolResult> {
        let script_dir = Path::new(workspace).join(PROMPT_DIR);
        tokio::fs::create_dir_all(&script_dir).await?;

        let prompt_file = script_dir.join("agent-prompt.txt");
        tokio::fs::write(&prompt_file, message).await?;

        let log_path = Path::new(workspace).join(AGENT_OUTPUT_LOG);
        let status_path = Path::new(workspace).join(AGENT_STATUS_FILE);

        // Build a shell script that tracks status and logs output
        let mut agent_cmd = String::from("cursor agent");
        if force {
            agent_cmd.push_str(" --force");
        }
        agent_cmd.push_str(&format!(" --workspace '{}'", workspace));
        if let Some(m) = model {
            agent_cmd.push_str(&format!(" --model '{}'", m));
        }
        agent_cmd.push_str(" \"$PROMPT\"");

        let script = format!(
            r#"#!/bin/bash
cd '{workspace}'
PROMPT=$(cat '{prompt_file}')
LOG='{log_path}'
STATUS='{status_path}'
START_TIME=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Record running status
printf '{{"status":"running","pid":%d,"started_at":"%s","prompt":"%s"}}\n' \
  $$ "$START_TIME" "$(echo "$PROMPT" | head -c 200 | tr '"' "'")" > "$STATUS"

# Clear previous log
> "$LOG"

echo "=== Cursor Agent started at $START_TIME ===" | tee -a "$LOG"
echo "=== Workspace: {workspace} ===" | tee -a "$LOG"
echo "=== Prompt: $(echo "$PROMPT" | head -c 200) ===" | tee -a "$LOG"
echo "" | tee -a "$LOG"

# Run agent and tee output to both terminal and log file
{agent_cmd} 2>&1 | tee -a "$LOG"
EXIT_CODE=${{PIPESTATUS[0]}}

END_TIME=$(date -u +%Y-%m-%dT%H:%M:%SZ)
if [ $EXIT_CODE -eq 0 ]; then
  FINAL_STATUS="completed"
else
  FINAL_STATUS="failed"
fi

printf '{{"status":"%s","pid":%d,"exit_code":%d,"started_at":"%s","finished_at":"%s"}}\n' \
  "$FINAL_STATUS" $$ $EXIT_CODE "$START_TIME" "$END_TIME" > "$STATUS"

echo "" | tee -a "$LOG"
echo "=== Agent $FINAL_STATUS at $END_TIME (exit code: $EXIT_CODE) ===" | tee -a "$LOG"
"#,
            workspace = workspace,
            prompt_file = prompt_file.display(),
            log_path = log_path.display(),
            status_path = status_path.display(),
            agent_cmd = agent_cmd,
        );

        let script_file = script_dir.join("agent-run.sh");
        tokio::fs::write(&script_file, &script).await?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(&script_file, std::fs::Permissions::from_mode(0o755))
                .await?;
        }

        let script_path_str = script_file.display().to_string();

        #[cfg(target_os = "macos")]
        {
            let apple_script = format!(
                "tell application \"Terminal\"\n\
                     activate\n\
                     do script \"bash '{}'\"\n\
                 end tell",
                script_path_str
            );
            let output = tokio::process::Command::new("osascript")
                .args(["-e", &apple_script])
                .output()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to open Terminal.app: {e}"))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("osascript failed: {stderr}");
            }
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let terminals = ["gnome-terminal", "konsole", "xfce4-terminal", "xterm"];
            let mut opened = false;
            for term in &terminals {
                let result = tokio::process::Command::new(term)
                    .args(["--", "bash", &script_path_str])
                    .spawn();
                if result.is_ok() {
                    opened = true;
                    break;
                }
            }
            if !opened {
                anyhow::bail!(
                    "No terminal emulator found. Run manually: bash '{}'",
                    script_path_str
                );
            }
        }

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&json!({
                "status": "started",
                "mode": "terminal",
                "workspace": workspace,
                "prompt_length": message.len(),
                "script": script_path_str,
                "log_file": log_path.display().to_string(),
                "status_file": status_path.display().to_string(),
                "model": model.unwrap_or("default"),
                "force": force,
                "hint": "Poll with {\"operation\":\"agent_status\",\"workspace\":\"...\"} to check progress and read output."
            }))
            .unwrap_or_default(),
            error: None,
        })
    }

    /// Reads agent status and output log for a workspace.
    async fn cmd_agent_status(&self, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
        let workspace = args
            .get("workspace")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.workspace_dir.display().to_string());
        let workspace = Self::sanitize_path(&workspace)?;

        let tail_lines = args.get("tail").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

        let status_path = Path::new(&workspace).join(AGENT_STATUS_FILE);
        let log_path = Path::new(&workspace).join(AGENT_OUTPUT_LOG);

        // Read status file
        let status_json: serde_json::Value = match tokio::fs::read_to_string(&status_path).await {
            Ok(content) => serde_json::from_str(&content).unwrap_or(json!({"status": "unknown"})),
            Err(_) => {
                return Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "status": "not_found",
                        "message": "No agent has been run in this workspace yet. Start one with {\"operation\":\"agent\",\"message\":\"...\"}",
                        "workspace": workspace,
                    }))
                    .unwrap_or_default(),
                    error: None,
                });
            }
        };

        // If status says "running", verify the PID is still alive
        let mut effective_status = status_json
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        if effective_status == "running" {
            if let Some(pid) = status_json.get("pid").and_then(|v| v.as_u64()) {
                let alive = tokio::process::Command::new("kill")
                    .args(["-0", &pid.to_string()])
                    .output()
                    .await
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                if !alive {
                    effective_status = "dead".to_string();
                }
            }
        }

        // Read tail of output log
        let output_tail = match tokio::fs::read_to_string(&log_path).await {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let start = if lines.len() > tail_lines {
                    lines.len() - tail_lines
                } else {
                    0
                };
                lines[start..].join("\n")
            }
            Err(_) => String::new(),
        };

        let log_total_lines = if output_tail.is_empty() {
            0
        } else {
            output_tail.lines().count()
        };

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&json!({
                "status": effective_status,
                "pid": status_json.get("pid"),
                "exit_code": status_json.get("exit_code"),
                "started_at": status_json.get("started_at"),
                "finished_at": status_json.get("finished_at"),
                "workspace": workspace,
                "log_file": log_path.display().to_string(),
                "output_lines": log_total_lines,
                "output_tail": output_tail,
            }))
            .unwrap_or_default(),
            error: None,
        })
    }

    /// Runs `cursor agent --print` headlessly and waits for the result.
    /// Use only for scripting or when you don't need a visible terminal.
    async fn run_agent_headless(
        &self,
        message: &str,
        workspace: &str,
        model: Option<&str>,
        force: bool,
        args: &serde_json::Value,
    ) -> anyhow::Result<ToolResult> {
        let output_format = args
            .get("output_format")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        let mut cli_args: Vec<String> = vec!["agent".into(), "--print".into()];
        cli_args.push("--workspace".into());
        cli_args.push(workspace.to_string());
        cli_args.push("--trust".into());
        cli_args.push("--output-format".into());
        cli_args.push(output_format.to_string());
        if force {
            cli_args.push("--force".into());
        }
        if let Some(m) = model {
            cli_args.push("--model".into());
            cli_args.push(m.to_string());
        }
        cli_args.push(message.to_string());

        let cli_refs: Vec<&str> = cli_args.iter().map(|s| s.as_str()).collect();

        let timeout = std::time::Duration::from_secs(AGENT_TIMEOUT_SECS);
        let output = tokio::time::timeout(timeout, async {
            tokio::process::Command::new(Self::cursor_bin())
                .args(&cli_refs)
                .current_dir(&self.workspace_dir)
                .output()
                .await
        })
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "Cursor agent timed out after {}s. Use headless=false to run in a terminal instead.",
                AGENT_TIMEOUT_SECS
            )
        })?
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "cursor CLI not found on PATH. Install: open Cursor IDE → Cmd+Shift+P → \
                     'Shell Command: Install cursor command in PATH'"
                )
            } else {
                anyhow::anyhow!("Failed to run cursor agent: {e}")
            }
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();
        let agent_output = if stdout.trim().is_empty() {
            stderr.trim().to_string()
        } else {
            stdout
        };

        if !success && agent_output.is_empty() {
            anyhow::bail!("Cursor agent exited with {}", output.status);
        }

        Ok(ToolResult {
            success,
            output: serde_json::to_string_pretty(&json!({
                "mode": "headless",
                "workspace": workspace,
                "model": model.unwrap_or("default"),
                "force": force,
                "exit_code": output.status.code(),
                "output": agent_output,
            }))
            .unwrap_or_else(|_| agent_output.clone()),
            error: if success { None } else { Some(agent_output) },
        })
    }

    fn cmd_help() -> ToolResult {
        let guide = r#"{
  "tool": "cursor_cli",
  "prerequisite": "Install the `cursor` CLI: open Cursor IDE → Cmd+Shift+P → 'Shell Command: Install cursor command in PATH'",
  "operations": {
    "open": {
      "description": "Open a file or folder in Cursor",
      "required_params": [],
      "optional_params": ["path (default: '.')", "reuse_window (default: true)", "wait (default: false)"],
      "examples": [
        {"operation": "open", "path": "src/main.rs"},
        {"operation": "open", "path": "/absolute/project/dir", "reuse_window": false}
      ]
    },
    "goto": {
      "description": "Open a file and jump to a specific line and column",
      "required_params": ["file"],
      "optional_params": ["line (default: 1)", "column (default: 1)"],
      "examples": [
        {"operation": "goto", "file": "src/lib.rs", "line": 42},
        {"operation": "goto", "file": "src/lib.rs", "line": 100, "column": 15}
      ]
    },
    "diff": {
      "description": "Open a side-by-side diff of two files",
      "required_params": ["file1", "file2"],
      "examples": [
        {"operation": "diff", "file1": "old_config.toml", "file2": "new_config.toml"}
      ]
    },
    "prompt": {
      "description": "Send a prompt/instruction to Cursor IDE — writes it to .cursor/agent-prompt.md and opens it in the GUI. Use 'agent' operation instead if you want headless CLI execution.",
      "required_params": ["message"],
      "optional_params": ["file (target file for context)", "mode ('composer'|'chat'|'edit'|'rules', default: 'composer')"],
      "examples": [
        {"operation": "prompt", "message": "Refactor this function to use async/await", "file": "src/handler.rs", "mode": "composer"},
        {"operation": "prompt", "message": "Explain the error handling strategy in this module", "mode": "chat"},
        {"operation": "prompt", "message": "Always use Result<T> instead of unwrap() in production code", "mode": "rules"}
      ]
    },
    "agent": {
      "description": "Run Cursor Agent CLI (no IDE needed). By default opens a visible Terminal window with the agent running interactively. Set headless=true for scripted/background execution with --print.",
      "required_params": ["message"],
      "optional_params": ["workspace (default: current)", "model (e.g. 'sonnet-4', 'gpt-5')", "force (auto-approve, default: true)", "headless (default: false — opens terminal window)", "output_format (for headless: 'text'|'json'|'stream-json')"],
      "examples": [
        {"operation": "agent", "message": "Add unit tests for the auth module", "workspace": "/path/to/project"},
        {"operation": "agent", "message": "Review all Go files and fix linting issues", "model": "sonnet-4"},
        {"operation": "agent", "message": "Explain the architecture", "force": false},
        {"operation": "agent", "message": "Summarize this project", "headless": true}
      ]
    },
    "agent_status": {
      "description": "Check the status and output of a running (or completed) Cursor Agent. Returns status (running/completed/failed/dead/not_found), PID, exit code, timestamps, and tail of the output log.",
      "required_params": [],
      "optional_params": ["workspace (default: current)", "tail (number of output lines, default: 50)"],
      "examples": [
        {"operation": "agent_status"},
        {"operation": "agent_status", "workspace": "/path/to/project"},
        {"operation": "agent_status", "workspace": "/path/to/project", "tail": 100}
      ]
    },
    "install_extension": {
      "description": "Install a VS Code / Cursor extension by marketplace ID",
      "required_params": ["extension_id"],
      "examples": [
        {"operation": "install_extension", "extension_id": "ms-python.python"},
        {"operation": "install_extension", "extension_id": "rust-lang.rust-analyzer"}
      ]
    },
    "uninstall_extension": {
      "description": "Uninstall an extension by marketplace ID",
      "required_params": ["extension_id"],
      "examples": [
        {"operation": "uninstall_extension", "extension_id": "ms-python.python"}
      ]
    },
    "list_extensions": {
      "description": "List all installed Cursor extensions",
      "required_params": [],
      "examples": [
        {"operation": "list_extensions"}
      ]
    },
    "new_window": {
      "description": "Open a new Cursor window, optionally at a specific path",
      "required_params": [],
      "optional_params": ["path"],
      "examples": [
        {"operation": "new_window"},
        {"operation": "new_window", "path": "/other/project"}
      ]
    },
    "status": {
      "description": "Report Cursor version, installed extensions, and workspace info",
      "required_params": [],
      "examples": [
        {"operation": "status"}
      ]
    },
    "help": {
      "description": "Show this usage guide with examples for every operation",
      "required_params": [],
      "examples": [
        {"operation": "help"}
      ]
    }
  }
}"#;

        ToolResult {
            success: true,
            output: guide.to_string(),
            error: None,
        }
    }
}

#[async_trait]
impl Tool for CursorCliTool {
    fn name(&self) -> &str {
        "cursor_cli"
    }

    fn description(&self) -> &str {
        "Control Cursor via its CLI. Key operation: 'agent' runs Cursor Agent headlessly in \
         terminal (no IDE needed) — it reads, writes, and runs shell commands on the workspace \
         autonomously. Other operations: open, goto, diff, prompt, install_extension, \
         uninstall_extension, list_extensions, new_window, status, help. \
         Requires the `cursor` command on PATH. Call with {\"operation\":\"help\"} for full usage."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": [
                        "open",
                        "goto",
                        "diff",
                        "prompt",
                        "agent",
                        "agent_status",
                        "install_extension",
                        "uninstall_extension",
                        "list_extensions",
                        "new_window",
                        "status",
                        "help"
                    ],
                    "description": "Cursor CLI operation. 'agent' starts Cursor Agent in a terminal. 'agent_status' polls agent progress and reads output. Use 'help' for full guide."
                },
                "path": {
                    "type": "string",
                    "description": "File or folder path (for 'open' and 'new_window')"
                },
                "file": {
                    "type": "string",
                    "description": "File path (for 'goto')"
                },
                "line": {
                    "type": "integer",
                    "description": "Line number (for 'goto', default: 1)"
                },
                "column": {
                    "type": "integer",
                    "description": "Column number (for 'goto', default: 1)"
                },
                "file1": {
                    "type": "string",
                    "description": "First file path (for 'diff')"
                },
                "file2": {
                    "type": "string",
                    "description": "Second file path (for 'diff')"
                },
                "message": {
                    "type": "string",
                    "description": "Prompt or instruction text to send to Cursor AI (for 'prompt')"
                },
                "mode": {
                    "type": "string",
                    "enum": ["composer", "chat", "edit", "rules"],
                    "description": "How to use the prompt in Cursor (for 'prompt'): 'composer' = Cmd+I multi-file edits, 'chat' = Cmd+L ask questions, 'edit' = Cmd+K inline code edits, 'rules' = save as project rule. Default: 'composer'"
                },
                "extension_id": {
                    "type": "string",
                    "description": "Extension marketplace ID, e.g. 'ms-python.python' (for install/uninstall)"
                },
                "reuse_window": {
                    "type": "boolean",
                    "description": "Reuse existing window instead of opening a new one (for 'open', default: true)"
                },
                "wait": {
                    "type": "boolean",
                    "description": "Wait for the file to be closed before returning (for 'open', default: false)"
                },
                "model": {
                    "type": "string",
                    "description": "AI model to use (for 'agent'), e.g. 'gpt-5', 'sonnet-4', 'sonnet-4-thinking'"
                },
                "output_format": {
                    "type": "string",
                    "enum": ["text", "json", "stream-json"],
                    "description": "Output format for agent responses (for 'agent', default: 'text')"
                },
                "force": {
                    "type": "boolean",
                    "description": "Auto-approve all agent actions without prompting (for 'agent', default: true)"
                },
                "headless": {
                    "type": "boolean",
                    "description": "Run agent in headless mode (--print, no terminal window). Default: false (opens a visible terminal)"
                },
                "workspace": {
                    "type": "string",
                    "description": "Workspace directory for agent to operate on (for 'agent'/'agent_status', defaults to current workspace)"
                },
                "tail": {
                    "type": "integer",
                    "description": "Number of output lines to return (for 'agent_status', default: 50)"
                }
            },
            "required": ["operation"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let operation = match args.get("operation").and_then(|v| v.as_str()) {
            Some(op) => op,
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'operation' parameter".into()),
                });
            }
        };

        if Self::requires_write_access(operation) {
            if !self.security.can_act() {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(
                        "Action blocked: this Cursor CLI operation requires higher autonomy level"
                            .into(),
                    ),
                });
            }
            if matches!(self.security.autonomy, AutonomyLevel::ReadOnly) {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Action blocked: read-only mode".into()),
                });
            }
        }

        if !self.security.record_action() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Action blocked: rate limit exceeded".into()),
            });
        }

        match operation {
            "open" => self.cmd_open(&args).await,
            "goto" => self.cmd_goto(&args).await,
            "diff" => self.cmd_diff(&args).await,
            "prompt" => self.cmd_prompt(&args).await,
            "agent" => self.cmd_agent(&args).await,
            "agent_status" => self.cmd_agent_status(&args).await,
            "install_extension" => self.cmd_install_extension(&args).await,
            "uninstall_extension" => self.cmd_uninstall_extension(&args).await,
            "list_extensions" => self.cmd_list_extensions().await,
            "new_window" => self.cmd_new_window(&args).await,
            "status" => self.cmd_status().await,
            "help" => Ok(Self::cmd_help()),
            _ => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "Unknown cursor_cli operation: {operation}. \
                     Use {{\"operation\":\"help\"}} for a list of all operations."
                )),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{AutonomyLevel, SecurityPolicy};
    use tempfile::TempDir;

    fn test_tool(dir: &std::path::Path) -> CursorCliTool {
        let security = Arc::new(SecurityPolicy {
            autonomy: AutonomyLevel::Supervised,
            ..SecurityPolicy::default()
        });
        CursorCliTool::new(security, dir.to_path_buf())
    }

    #[test]
    fn sanitize_path_blocks_metacharacters() {
        assert!(CursorCliTool::sanitize_path("$(echo pwned)").is_err());
        assert!(CursorCliTool::sanitize_path("`rm -rf /`").is_err());
        assert!(CursorCliTool::sanitize_path("file | cat").is_err());
        assert!(CursorCliTool::sanitize_path("file; rm x").is_err());
        assert!(CursorCliTool::sanitize_path("file > /tmp/out").is_err());
        assert!(CursorCliTool::sanitize_path("file < /etc/passwd").is_err());
        assert!(CursorCliTool::sanitize_path("file & bg").is_err());
    }

    #[test]
    fn sanitize_path_allows_safe_paths() {
        assert!(CursorCliTool::sanitize_path("src/main.rs").is_ok());
        assert!(CursorCliTool::sanitize_path("/absolute/path/file.txt").is_ok());
        assert!(CursorCliTool::sanitize_path(".").is_ok());
        assert!(CursorCliTool::sanitize_path("path with spaces/file.rs").is_ok());
        assert!(CursorCliTool::sanitize_path("../relative/path").is_ok());
    }

    #[test]
    fn sanitize_extension_id_valid() {
        assert_eq!(
            CursorCliTool::sanitize_extension_id("ms-python.python").unwrap(),
            "ms-python.python"
        );
        assert_eq!(
            CursorCliTool::sanitize_extension_id("rust-lang.rust-analyzer").unwrap(),
            "rust-lang.rust-analyzer"
        );
        assert_eq!(
            CursorCliTool::sanitize_extension_id("bradlc.vscode-tailwindcss").unwrap(),
            "bradlc.vscode-tailwindcss"
        );
    }

    #[test]
    fn sanitize_extension_id_rejects_invalid() {
        assert!(CursorCliTool::sanitize_extension_id("").is_err());
        assert!(CursorCliTool::sanitize_extension_id("   ").is_err());
        assert!(CursorCliTool::sanitize_extension_id("ext id with spaces").is_err());
        assert!(CursorCliTool::sanitize_extension_id("ext;injection").is_err());
        assert!(CursorCliTool::sanitize_extension_id("$(cmd)").is_err());
    }

    #[test]
    fn requires_write_access_classification() {
        assert!(CursorCliTool::requires_write_access("install_extension"));
        assert!(CursorCliTool::requires_write_access("uninstall_extension"));
        assert!(CursorCliTool::requires_write_access("new_window"));
        assert!(CursorCliTool::requires_write_access("prompt"));
        assert!(CursorCliTool::requires_write_access("agent"));

        assert!(!CursorCliTool::requires_write_access("agent_status"));
        assert!(!CursorCliTool::requires_write_access("open"));
        assert!(!CursorCliTool::requires_write_access("goto"));
        assert!(!CursorCliTool::requires_write_access("diff"));
        assert!(!CursorCliTool::requires_write_access("list_extensions"));
        assert!(!CursorCliTool::requires_write_access("status"));
        assert!(!CursorCliTool::requires_write_access("help"));
    }

    #[test]
    fn tool_metadata() {
        let tmp = TempDir::new().unwrap();
        let tool = test_tool(tmp.path());
        assert_eq!(tool.name(), "cursor_cli");
        assert!(!tool.description().is_empty());
        let schema = tool.parameters_schema();
        assert!(schema["properties"]["operation"].is_object());
    }

    #[tokio::test]
    async fn blocks_readonly_for_extension_install() {
        let tmp = TempDir::new().unwrap();
        let security = Arc::new(SecurityPolicy {
            autonomy: AutonomyLevel::ReadOnly,
            ..SecurityPolicy::default()
        });
        let tool = CursorCliTool::new(security, tmp.path().to_path_buf());

        let result = tool
            .execute(json!({
                "operation": "install_extension",
                "extension_id": "ms-python.python"
            }))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result
            .error
            .as_deref()
            .unwrap_or("")
            .contains("higher autonomy"));
    }

    #[tokio::test]
    async fn rejects_missing_operation() {
        let tmp = TempDir::new().unwrap();
        let tool = test_tool(tmp.path());

        let result = tool.execute(json!({})).await.unwrap();
        assert!(!result.success);
        assert!(result
            .error
            .as_deref()
            .unwrap_or("")
            .contains("Missing 'operation'"));
    }

    #[tokio::test]
    async fn rejects_unknown_operation() {
        let tmp = TempDir::new().unwrap();
        let tool = test_tool(tmp.path());

        let result = tool.execute(json!({"operation": "reboot"})).await.unwrap();
        assert!(!result.success);
        assert!(result
            .error
            .as_deref()
            .unwrap_or("")
            .contains("Unknown cursor_cli operation"));
    }

    #[tokio::test]
    async fn blocks_rate_limited() {
        let tmp = TempDir::new().unwrap();
        let security = Arc::new(SecurityPolicy {
            max_actions_per_hour: 0,
            ..SecurityPolicy::default()
        });
        let tool = CursorCliTool::new(security, tmp.path().to_path_buf());

        let result = tool.execute(json!({"operation": "status"})).await.unwrap();
        assert!(!result.success);
        assert!(result.error.as_deref().unwrap_or("").contains("rate limit"));
    }

    #[tokio::test]
    async fn help_returns_guide() {
        let tmp = TempDir::new().unwrap();
        let tool = test_tool(tmp.path());

        let result = tool.execute(json!({"operation": "help"})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("prompt"));
        assert!(result.output.contains("agent"));
        assert!(result.output.contains("agent_status"));
        assert!(result.output.contains("open"));
        assert!(result.output.contains("goto"));
        assert!(result.output.contains("diff"));
        assert!(result.output.contains("install_extension"));
        assert!(result.output.contains("list_extensions"));
        assert!(result.output.contains("status"));
        assert!(result.output.contains("examples"));
    }

    #[test]
    fn schema_includes_agent_operations() {
        let tmp = TempDir::new().unwrap();
        let tool = test_tool(tmp.path());
        let schema = tool.parameters_schema();
        let ops = schema["properties"]["operation"]["enum"]
            .as_array()
            .unwrap();
        let op_names: Vec<&str> = ops.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(op_names.contains(&"agent"));
        assert!(op_names.contains(&"agent_status"));
        assert!(schema["properties"]["model"].is_object());
        assert!(schema["properties"]["output_format"].is_object());
        assert!(schema["properties"]["force"].is_object());
        assert!(schema["properties"]["headless"].is_object());
        assert!(schema["properties"]["workspace"].is_object());
        assert!(schema["properties"]["tail"].is_object());
    }

    #[tokio::test]
    async fn agent_status_not_found() {
        let tmp = TempDir::new().unwrap();
        let tool = test_tool(tmp.path());
        let result = tool
            .execute(json!({
                "operation": "agent_status",
                "workspace": tmp.path().to_str().unwrap()
            }))
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("not_found"));
    }

    #[tokio::test]
    async fn help_contains_usage_guidance() {
        let tmp = TempDir::new().unwrap();
        let tool = test_tool(tmp.path());

        let result = tool.execute(json!({"operation": "help"})).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result.output).unwrap();
        assert!(parsed["prerequisite"].as_str().unwrap().contains("cursor"));
        assert!(parsed["operations"]["prompt"]["examples"].is_array());
        assert!(parsed["operations"]["open"]["examples"].is_array());
    }

    #[tokio::test]
    async fn blocks_readonly_for_prompt() {
        let tmp = TempDir::new().unwrap();
        let security = Arc::new(SecurityPolicy {
            autonomy: AutonomyLevel::ReadOnly,
            ..SecurityPolicy::default()
        });
        let tool = CursorCliTool::new(security, tmp.path().to_path_buf());

        let result = tool
            .execute(json!({
                "operation": "prompt",
                "message": "Refactor this"
            }))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result
            .error
            .as_deref()
            .unwrap_or("")
            .contains("higher autonomy"));
    }

    #[tokio::test]
    async fn unknown_op_suggests_help() {
        let tmp = TempDir::new().unwrap();
        let tool = test_tool(tmp.path());

        let result = tool.execute(json!({"operation": "reboot"})).await.unwrap();
        assert!(!result.success);
        assert!(result.error.as_deref().unwrap_or("").contains("help"));
    }
}
