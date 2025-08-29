use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub groq_api_key: Option<String>,
    pub default_model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        
        Ok(())
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        
        Ok(home_dir.join(".config").join("rust-coding-agent").join("config.json"))
    }

    pub fn get_groq_key(&self) -> Option<String> {
        self.groq_api_key.clone()
    }

    pub fn set_groq_key(&mut self, key: String) -> Result<()> {
        self.groq_api_key = Some(key);
        self.save()
    }

    pub fn get_model(&self) -> &str {
        &self.default_model
    }

    pub fn set_model(&mut self, model: String) -> Result<()> {
        self.default_model = model;
        self.save()
    }

    pub fn get_temperature(&self) -> f32 {
        self.temperature
    }

    pub fn set_temperature(&mut self, temperature: f32) -> Result<()> {
        self.temperature = temperature.clamp(0.0, 2.0);
        self.save()
    }

    pub fn get_max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }

    pub fn set_max_tokens(&mut self, max_tokens: Option<u32>) -> Result<()> {
        self.max_tokens = max_tokens;
        self.save()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            groq_api_key: None,
            default_model: "llama-3.1-70b-versatile".to_string(),
            temperature: 0.7,
            max_tokens: Some(4096),
        }
    }
}