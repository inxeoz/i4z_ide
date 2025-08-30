use super::{AgentAction, AgentExecutor, AgentResponse};
use crate::api::GroqClient;
use anyhow::Result;
use regex::Regex;
use serde_json;
use std::path::PathBuf;

pub struct AgentActionParser;

impl AgentActionParser {
    pub fn parse_agent_response(response: &str) -> Vec<AgentAction> {
        let mut actions = Vec::new();

        // Look for action blocks in the response
        if let Some(actions_from_json) = Self::extract_json_actions(response) {
            actions.extend(actions_from_json);
        }

        // Look for natural language actions
        actions.extend(Self::extract_natural_language_actions(response));

        actions
    }

    fn extract_json_actions(response: &str) -> Option<Vec<AgentAction>> {
        // Look for JSON blocks containing actions
        let json_regex = Regex::new(r"```json\s*(.*?)\s*```").ok()?;
        
        for cap in json_regex.captures_iter(response) {
            if let Some(json_text) = cap.get(1) {
                if let Ok(actions) = serde_json::from_str::<Vec<AgentAction>>(json_text.as_str()) {
                    return Some(actions);
                }
                // Try parsing as a single action
                if let Ok(action) = serde_json::from_str::<AgentAction>(json_text.as_str()) {
                    return Some(vec![action]);
                }
            }
        }

        None
    }

    fn extract_natural_language_actions(response: &str) -> Vec<AgentAction> {
        let mut actions = Vec::new();

        // Look for common patterns that indicate file operations
        let patterns = [
            (r#"(?i)read\s+(?:the\s+)?file\s+[`"']?([^`"'\s]+)[`"']?"#, "read"),
            (r#"(?i)write\s+(?:to\s+)?(?:the\s+)?file\s+[`"']?([^`"'\s]+)[`"']?"#, "write"),
            (r#"(?i)create\s+(?:a\s+)?(?:new\s+)?file(?:\s+called)?\s+[`\"']?([^`\"'\s]+)[`\"']?\"#, "write"),
            (r#"(?i)save\s+(?:to\s+)?[`"']?([^`"'\s]+)[`"']?"#, "write"),
            (r#"(?i)delete\s+(?:the\s+)?file\s+[`"']?([^`"'\s]+)[`"']?"#, "delete"),
            (r#"(?i)remove\s+(?:the\s+)?file\s+[`"']?([^`"'\s]+)[`"']?"#, "delete"),
            (r#"(?i)list\s+(?:the\s+)?(?:files\s+in\s+)?(?:directory\s+)?[`"']?([^`"'\s]+)[`"']?"#, "list"),
            (r#"(?i)execute\s+[`"']?([^`"'\n]+)[`"']?"#, "execute"),
            (r#"(?i)run\s+[`"']?([^`"'\n]+)[`"']?"#, "execute"),
        ];

        for (pattern, action_type) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                for cap in regex.captures_iter(response) {
                    if let Some(target) = cap.get(1) {
                        let target_str = target.as_str().trim();
                        
                        let action = match action_type {
                            "read" => Some(AgentAction::ReadFile { 
                                path: PathBuf::from(target_str) 
                            }),
                            "write" => {
                                // For write actions, we need to extract content from context
                                // This is a simplified approach
                                Some(AgentAction::WriteFile { 
                                    path: PathBuf::from(target_str),
                                    content: Self::extract_content_for_file(response, target_str)
                                        .unwrap_or_else(|| "// TODO: Add content".to_string())
                                })
                            },
                            "delete" => Some(AgentAction::DeleteFile { 
                                path: PathBuf::from(target_str) 
                            }),
                            "list" => Some(AgentAction::ListDirectory { 
                                path: PathBuf::from(target_str) 
                            }),
                            "execute" => Some(AgentAction::ExecuteCommand { 
                                command: target_str.to_string(),
                                working_dir: None
                            }),
                            _ => None,
                        };

                        if let Some(action) = action {
                            actions.push(action);
                        }
                    }
                }
            }
        }

        actions
    }

    fn extract_content_for_file(response: &str, filename: &str) -> Option<String> {
        // Look for code blocks near the filename mention
        let code_block_regex = Regex::new(r"```(?:\w+)?\s*(.*?)\s*```").ok()?;
        
        // Find the position of the filename in the response
        let filename_pos = response.find(filename)?;
        
        // Look for code blocks around the filename mention
        for cap in code_block_regex.captures_iter(response) {
            if let Some(code_match) = cap.get(0) {
                let code_start = code_match.start();
                let code_end = code_match.end();
                
                // If the code block is within 500 characters of the filename mention
                if (code_start as i32 - filename_pos as i32).abs() < 500 {
                    if let Some(content) = cap.get(1) {
                        return Some(content.as_str().to_string());
                    }
                }
            }
        }

        None
    }
}

pub async fn process_agent_message(
    message: &str,
    executor: &mut dyn AgentExecutor,
) -> Result<Vec<AgentResponse>> {
    let actions = AgentActionParser::parse_agent_response(message);
    let mut responses = Vec::new();

    for action in actions {
        let response = executor.execute_action(action)?;
        responses.push(response);
    }

    Ok(responses)
}

pub fn format_agent_responses(responses: &[AgentResponse]) -> String {
    if responses.is_empty() {
        return "No actions were executed.".to_string();
    }

    let mut output = String::new();
    output.push_str("🤖 Agent Actions Executed:\n\n");

    for (i, response) in responses.iter().enumerate() {
        let status_icon = if response.success { "✅" } else { "❌" };
        output.push_str(&format!("{}. {} {}\n", i + 1, status_icon, response.message));
        
        if let Some(data) = &response.data {
            if !data.is_empty() {
                output.push_str("   Output:\n");
                for line in data.lines().take(10) { // Limit output lines
                    output.push_str(&format!("   {}\n", line));
                }
                if data.lines().count() > 10 {
                    output.push_str("   ... (output truncated)\n");
                }
            }
        }

        if let Some(error) = &response.error {
            output.push_str(&format!("   Error: {}\n", error));
        }

        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_natural_language_actions() {
        let response = "I need to read the file src/main.rs and then create a new file called test.txt";
        let actions = AgentActionParser::extract_natural_language_actions(response);
        
        assert_eq!(actions.len(), 2);
        
        match &actions[0] {
            AgentAction::ReadFile { path } => {
                assert_eq!(path, &PathBuf::from("src/main.rs"));
            }
            _ => panic!("Expected ReadFile action"),
        }

        match &actions[1] {
            AgentAction::WriteFile { path, .. } => {
                assert_eq!(path, &PathBuf::from("test.txt"));
            }
            _ => panic!("Expected WriteFile action"),
        }
    }
}