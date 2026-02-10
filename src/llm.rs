use anyhow::{Context, Result};
use serde_json::json;
use std::env;

#[derive(Clone)]
pub enum Provider {
    OpenAI,
    Ollama,
}

pub struct LLMClient {
    provider: Provider,
    model: String,
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl LLMClient {
    pub fn new(provider_str: String, model: Option<String>) -> Self {
        let provider = match provider_str.to_lowercase().as_str() {
            "ollama" => Provider::Ollama,
            _ => Provider::OpenAI,
        };

        let (default_model, api_key, base_url) = match provider {
            Provider::OpenAI => (
                "gpt-4o".to_string(),
                env::var("OPENAI_API_KEY").unwrap_or_default(),
                "https://api.openai.com/v1/chat/completions".to_string(),
            ),
            Provider::Ollama => (
                "llama3".to_string(),
                "ollama".to_string(),
                "http://localhost:11434/api/chat".to_string(),
            ),
        };

        LLMClient {
            provider,
            model: model.unwrap_or(default_model),
            api_key,
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn chat(&self, system: &str, user: &str) -> Result<String> {
        let body = match self.provider {
            Provider::OpenAI => json!({
                "model": self.model,
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": user }
                ],
                "response_format": { "type": "json_object" } // Force JSON for tool usage
            }),
            Provider::Ollama => json!({
                "model": self.model,
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": user }
                ],
                "format": "json",
                "stream": false
            }),
        };

        let mut request = self.client.post(&self.base_url).json(&body);

        if let Provider::OpenAI = self.provider {
            request = request.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let resp = request.send().await.context("Failed to send LLM request")?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "LLM Request failed: {:?}",
                resp.text().await?
            ));
        }

        let json: serde_json::Value = resp.json().await?;

        let content = match self.provider {
            Provider::OpenAI => json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            Provider::Ollama => json["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string(),
        };

        Ok(content)
    }
}
