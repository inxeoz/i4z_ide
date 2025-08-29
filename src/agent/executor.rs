use super::{AgentAction, AgentExecutor, AgentResponse, AgentCapabilities};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct DefaultAgentExecutor {
    pub capabilities: AgentCapabilities,
    pub current_directory: PathBuf,
}

impl DefaultAgentExecutor {
    pub fn new(current_directory: PathBuf) -> Self {
        Self {
            capabilities: AgentCapabilities::default(),
            current_directory,
        }
    }

    pub fn with_capabilities(mut self, capabilities: AgentCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    fn is_path_restricted(&self, path: &PathBuf) -> bool {
        for restricted in &self.capabilities.restricted_paths {
            if path.starts_with(restricted) {
                return true;
            }
        }
        false
    }

    fn resolve_path(&self, path: &PathBuf) -> PathBuf {
        if path.is_absolute() {
            path.clone()
        } else {
            self.current_directory.join(path)
        }
    }
}

impl AgentExecutor for DefaultAgentExecutor {
    fn execute_action(&mut self, action: AgentAction) -> Result<AgentResponse> {
        if !self.is_safe_action(&action) {
            return Ok(AgentResponse::error(
                "Action not permitted".to_string(),
                "This action is restricted by the current capabilities".to_string(),
            ));
        }

        match action {
            AgentAction::ReadFile { path } => {
                let resolved_path = self.resolve_path(&path);
                match fs::read_to_string(&resolved_path) {
                    Ok(content) => Ok(AgentResponse::success(
                        format!("Successfully read file: {}", resolved_path.display()),
                        Some(content),
                    )),
                    Err(e) => Ok(AgentResponse::error(
                        format!("Failed to read file: {}", resolved_path.display()),
                        e.to_string(),
                    )),
                }
            }

            AgentAction::WriteFile { path, content } => {
                let resolved_path = self.resolve_path(&path);
                
                // Create parent directories if they don't exist
                if let Some(parent) = resolved_path.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        return Ok(AgentResponse::error(
                            "Failed to create parent directories".to_string(),
                            e.to_string(),
                        ));
                    }
                }

                match fs::write(&resolved_path, content) {
                    Ok(_) => Ok(AgentResponse::success(
                        format!("Successfully wrote file: {}", resolved_path.display()),
                        None,
                    )),
                    Err(e) => Ok(AgentResponse::error(
                        format!("Failed to write file: {}", resolved_path.display()),
                        e.to_string(),
                    )),
                }
            }

            AgentAction::CreateDirectory { path } => {
                let resolved_path = self.resolve_path(&path);
                match fs::create_dir_all(&resolved_path) {
                    Ok(_) => Ok(AgentResponse::success(
                        format!("Successfully created directory: {}", resolved_path.display()),
                        None,
                    )),
                    Err(e) => Ok(AgentResponse::error(
                        format!("Failed to create directory: {}", resolved_path.display()),
                        e.to_string(),
                    )),
                }
            }

            AgentAction::DeleteFile { path } => {
                let resolved_path = self.resolve_path(&path);
                let result = if resolved_path.is_file() {
                    fs::remove_file(&resolved_path)
                } else if resolved_path.is_dir() {
                    fs::remove_dir_all(&resolved_path)
                } else {
                    return Ok(AgentResponse::error(
                        "Path does not exist".to_string(),
                        format!("Path {} does not exist", resolved_path.display()),
                    ));
                };

                match result {
                    Ok(_) => Ok(AgentResponse::success(
                        format!("Successfully deleted: {}", resolved_path.display()),
                        None,
                    )),
                    Err(e) => Ok(AgentResponse::error(
                        format!("Failed to delete: {}", resolved_path.display()),
                        e.to_string(),
                    )),
                }
            }

            AgentAction::ExecuteCommand { command, working_dir } => {
                let working_dir = working_dir.unwrap_or_else(|| self.current_directory.clone());
                let mut cmd = if cfg!(target_os = "windows") {
                    let mut cmd = Command::new("cmd");
                    cmd.args(["/C", &command]);
                    cmd
                } else {
                    let mut cmd = Command::new("sh");
                    cmd.args(["-c", &command]);
                    cmd
                };

                cmd.current_dir(&working_dir);

                match cmd.output() {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        let combined_output = if stderr.is_empty() {
                            stdout
                        } else {
                            format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr)
                        };

                        if output.status.success() {
                            Ok(AgentResponse::success(
                                format!("Command executed successfully: {}", command),
                                Some(combined_output),
                            ))
                        } else {
                            Ok(AgentResponse::error(
                                format!("Command failed: {}", command),
                                combined_output,
                            ))
                        }
                    }
                    Err(e) => Ok(AgentResponse::error(
                        format!("Failed to execute command: {}", command),
                        e.to_string(),
                    )),
                }
            }

            AgentAction::SearchFiles { pattern, directory } => {
                let search_dir = directory.unwrap_or_else(|| self.current_directory.clone());
                let resolved_dir = self.resolve_path(&search_dir);

                let mut matches = Vec::new();
                if let Ok(entries) = fs::read_dir(&resolved_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            if filename.contains(&pattern) {
                                matches.push(path.display().to_string());
                            }
                        }
                    }
                }

                Ok(AgentResponse::success(
                    format!("Found {} matches for pattern '{}'", matches.len(), pattern),
                    Some(matches.join("\n")),
                ))
            }

            AgentAction::ReplaceInFile { path, old, new } => {
                let resolved_path = self.resolve_path(&path);
                match fs::read_to_string(&resolved_path) {
                    Ok(content) => {
                        let new_content = content.replace(&old, &new);
                        match fs::write(&resolved_path, new_content) {
                            Ok(_) => Ok(AgentResponse::success(
                                format!("Successfully replaced text in: {}", resolved_path.display()),
                                None,
                            )),
                            Err(e) => Ok(AgentResponse::error(
                                format!("Failed to write file: {}", resolved_path.display()),
                                e.to_string(),
                            )),
                        }
                    }
                    Err(e) => Ok(AgentResponse::error(
                        format!("Failed to read file: {}", resolved_path.display()),
                        e.to_string(),
                    )),
                }
            }

            AgentAction::ListDirectory { path } => {
                let resolved_path = self.resolve_path(&path);
                match fs::read_dir(&resolved_path) {
                    Ok(entries) => {
                        let mut items = Vec::new();
                        for entry in entries.flatten() {
                            let path = entry.path();
                            let file_type = if path.is_dir() { "DIR" } else { "FILE" };
                            let name = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown");
                            items.push(format!("{:<6} {}", file_type, name));
                        }
                        items.sort();

                        Ok(AgentResponse::success(
                            format!("Listed directory: {}", resolved_path.display()),
                            Some(items.join("\n")),
                        ))
                    }
                    Err(e) => Ok(AgentResponse::error(
                        format!("Failed to list directory: {}", resolved_path.display()),
                        e.to_string(),
                    )),
                }
            }

            AgentAction::GetFileInfo { path } => {
                let resolved_path = self.resolve_path(&path);
                match fs::metadata(&resolved_path) {
                    Ok(metadata) => {
                        let file_type = if metadata.is_dir() {
                            "Directory"
                        } else if metadata.is_file() {
                            "File"
                        } else {
                            "Other"
                        };

                        let size = if metadata.is_file() {
                            format!("{} bytes", metadata.len())
                        } else {
                            "N/A".to_string()
                        };

                        let info = format!(
                            "Path: {}\nType: {}\nSize: {}\nReadonly: {}",
                            resolved_path.display(),
                            file_type,
                            size,
                            metadata.permissions().readonly()
                        );

                        Ok(AgentResponse::success(
                            format!("File info for: {}", resolved_path.display()),
                            Some(info),
                        ))
                    }
                    Err(e) => Ok(AgentResponse::error(
                        format!("Failed to get file info: {}", resolved_path.display()),
                        e.to_string(),
                    )),
                }
            }
        }
    }

    fn is_safe_action(&self, action: &AgentAction) -> bool {
        match action {
            AgentAction::ReadFile { path } => {
                self.capabilities.can_read_files && !self.is_path_restricted(&self.resolve_path(path))
            }
            AgentAction::WriteFile { path, .. } => {
                self.capabilities.can_write_files && !self.is_path_restricted(&self.resolve_path(path))
            }
            AgentAction::CreateDirectory { path } => {
                self.capabilities.can_modify_filesystem && !self.is_path_restricted(&self.resolve_path(path))
            }
            AgentAction::DeleteFile { path } => {
                self.capabilities.can_modify_filesystem && !self.is_path_restricted(&self.resolve_path(path))
            }
            AgentAction::ExecuteCommand { .. } => {
                self.capabilities.can_execute_commands
            }
            AgentAction::SearchFiles { directory, .. } => {
                if let Some(dir) = directory {
                    !self.is_path_restricted(&self.resolve_path(dir))
                } else {
                    !self.is_path_restricted(&self.current_directory)
                }
            }
            AgentAction::ReplaceInFile { path, .. } => {
                self.capabilities.can_write_files && !self.is_path_restricted(&self.resolve_path(path))
            }
            AgentAction::ListDirectory { path } => {
                self.capabilities.can_read_files && !self.is_path_restricted(&self.resolve_path(path))
            }
            AgentAction::GetFileInfo { path } => {
                self.capabilities.can_read_files && !self.is_path_restricted(&self.resolve_path(path))
            }
        }
    }
}