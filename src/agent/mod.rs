pub mod actions;
pub mod executor;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentAction {
    ReadFile { path: PathBuf },
    WriteFile { path: PathBuf, content: String },
    CreateDirectory { path: PathBuf },
    DeleteFile { path: PathBuf },
    ExecuteCommand { command: String, working_dir: Option<PathBuf> },
    SearchFiles { pattern: String, directory: Option<PathBuf> },
    ReplaceInFile { path: PathBuf, old: String, new: String },
    ListDirectory { path: PathBuf },
    GetFileInfo { path: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<String>,
    pub error: Option<String>,
}

impl AgentResponse {
    pub fn success(message: String, data: Option<String>) -> Self {
        Self {
            success: true,
            message,
            data,
            error: None,
        }
    }

    pub fn error(message: String, error: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
            error: Some(error),
        }
    }
}

pub trait AgentExecutor {
    fn execute_action(&mut self, action: AgentAction) -> Result<AgentResponse>;
    fn is_safe_action(&self, action: &AgentAction) -> bool;
}

pub struct AgentCapabilities {
    pub can_read_files: bool,
    pub can_write_files: bool,
    pub can_execute_commands: bool,
    pub can_modify_filesystem: bool,
    pub restricted_paths: Vec<PathBuf>,
}

impl Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            can_read_files: true,
            can_write_files: true,
            can_execute_commands: false, // Disabled by default for safety
            can_modify_filesystem: true,
            restricted_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/root"),
                PathBuf::from("/sys"),
                PathBuf::from("/proc"),
            ],
        }
    }
}