//! DatumBridge master policy — constraints OctoClaw (slave) must enforce.
//!
//! Option A: Policy is carried in WebSocket message payload.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Policy constraints from DatumBridge (master) that OctoClaw must enforce.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatumbridgePolicy {
    /// Only these tools are available. If empty, all tools except forbidden are allowed.
    #[serde(default, alias = "allowed_tools")]
    pub allowed_tools: Vec<String>,

    /// These tools are always forbidden.
    #[serde(default, alias = "forbidden_tools")]
    pub forbidden_tools: Vec<String>,

    /// File/shell tools only operate under these paths (prefix match). Empty = no restriction.
    #[serde(default, alias = "allowed_paths")]
    pub allowed_paths: Vec<String>,

    /// Block access to these paths (prefix match).
    #[serde(default, alias = "forbidden_paths")]
    pub forbidden_paths: Vec<String>,

    /// Stop after N tool invocations.
    #[serde(default, alias = "max_tool_calls")]
    pub max_tool_calls: Option<u32>,

    /// Max token cap for LLM response (if provider supports).
    #[serde(default, alias = "max_tokens")]
    pub max_tokens: Option<u32>,

    /// Timeout in seconds for the entire request.
    #[serde(default, alias = "timeout_secs")]
    pub timeout_secs: Option<u64>,

    /// Tools that require master approval before execution.
    #[serde(default, alias = "require_approval")]
    pub require_approval: Vec<String>,
}

impl DatumbridgePolicy {
    /// Returns true if the policy has any constraints (non-empty).
    pub fn is_active(&self) -> bool {
        !self.allowed_tools.is_empty()
            || !self.forbidden_tools.is_empty()
            || !self.allowed_paths.is_empty()
            || !self.forbidden_paths.is_empty()
            || self.max_tool_calls.is_some()
            || self.max_tokens.is_some()
            || self.timeout_secs.is_some()
            || !self.require_approval.is_empty()
    }

    /// Compute excluded tool names for the agent loop.
    /// Tools to exclude = forbidden_tools + (if allowed_tools non-empty) tools not in allowed_tools.
    pub fn excluded_tools(&self, all_tool_names: &[String]) -> Vec<String> {
        let forbidden: HashSet<String> = self
            .forbidden_tools
            .iter()
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        let allowed: HashSet<String> = self
            .allowed_tools
            .iter()
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        let mut excluded: Vec<String> = all_tool_names
            .iter()
            .filter(|name| {
                let n = name.trim().to_ascii_lowercase();
                if forbidden.contains(&n) {
                    return true;
                }
                if !allowed.is_empty() && !allowed.contains(&n) {
                    return true;
                }
                false
            })
            .cloned()
            .collect();

        excluded.sort();
        excluded.dedup();
        excluded
    }

    /// Returns true if the tool requires master approval.
    pub fn requires_approval(&self, tool_name: &str) -> bool {
        let n = tool_name.trim().to_ascii_lowercase();
        self.require_approval
            .iter()
            .any(|s| s.trim().to_ascii_lowercase() == n)
    }

    /// Check if a path is allowed. Returns `Some(false)` if forbidden, `Some(true)` if allowed,
    /// `None` if path checks are not configured.
    pub fn check_path(&self, path: &str) -> Option<bool> {
        let path = path.trim();
        if path.is_empty() {
            return Some(true);
        }

        // Normalize for comparison (resolve .. etc. in real impl would need path normalization)
        let path_lower = path.to_ascii_lowercase();

        for forbidden in &self.forbidden_paths {
            let p = forbidden.trim();
            if p.is_empty() {
                continue;
            }
            let fp = p.to_ascii_lowercase();
            if path_lower == fp || path_lower.starts_with(&format!("{fp}/")) {
                return Some(false);
            }
        }

        if self.allowed_paths.is_empty() {
            return None; // No path restriction
        }

        for allowed in &self.allowed_paths {
            let p = allowed.trim();
            if p.is_empty() {
                continue;
            }
            let ap = p.to_ascii_lowercase();
            if path_lower == ap || path_lower.starts_with(&format!("{ap}/")) {
                return Some(true);
            }
        }

        Some(false) // Not in allowed list
    }

    /// Max tool iterations for agent loop (from max_tool_calls).
    pub fn max_tool_iterations(&self, default: usize) -> usize {
        self.max_tool_calls.map(|n| n as usize).unwrap_or(default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excluded_tools_forbidden() {
        let policy = DatumbridgePolicy {
            forbidden_tools: vec!["shell".into(), "file_write".into()],
            ..Default::default()
        };
        let all = ["shell", "file_read", "memory_store", "file_write"]
            .map(String::from)
            .to_vec();
        let out = policy.excluded_tools(&all);
        assert!(out.contains(&"shell".to_string()));
        assert!(out.contains(&"file_write".to_string()));
        assert!(!out.contains(&"file_read".to_string()));
    }

    #[test]
    fn excluded_tools_allowed_only() {
        let policy = DatumbridgePolicy {
            allowed_tools: vec!["file_read".into(), "memory".into()],
            ..Default::default()
        };
        let all = ["shell", "file_read", "memory_store", "memory_recall"]
            .map(String::from)
            .to_vec();
        let out = policy.excluded_tools(&all);
        assert!(out.contains(&"shell".to_string()));
        assert!(!out.contains(&"file_read".to_string()));
        assert!(!out.contains(&"memory_store".to_string()));
    }

    #[test]
    fn check_path_forbidden() {
        let policy = DatumbridgePolicy {
            forbidden_paths: vec!["/etc/passwd".into(), "/root".into()],
            ..Default::default()
        };
        assert_eq!(policy.check_path("/etc/passwd"), Some(false));
        assert_eq!(policy.check_path("/root/.ssh"), Some(false));
    }

    #[test]
    fn check_path_allowed() {
        let policy = DatumbridgePolicy {
            allowed_paths: vec!["/var/log".into()],
            ..Default::default()
        };
        assert_eq!(policy.check_path("/var/log/syslog"), Some(true));
        assert_eq!(policy.check_path("/etc/passwd"), Some(false));
    }
}
