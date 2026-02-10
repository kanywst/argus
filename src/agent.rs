use crate::llm::LLMClient;
use crate::tools::ToolRunner;
use anyhow::Result;
use colored::*;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
enum AgentAction {
    #[serde(rename = "run_nikto")]
    RunNikto { target: String },
    #[serde(rename = "run_gobuster")]
    RunGobuster { target: String, wordlist: String },
    #[serde(rename = "finish")]
    Finish { summary: String },
}

pub struct Agent {
    client: LLMClient,
}

impl Agent {
    pub fn new(provider: String, model: Option<String>) -> Self {
        Agent {
            client: LLMClient::new(provider, model),
        }
    }

    pub async fn run_loop(
        &self,
        target: &str,
        wordlist: &str,
        output_dir: &Path,
        initial_context: &str,
    ) -> Result<()> {
        let system_prompt = r#"You are an expert Network Reconnaissance Agent. 
Your goal is to investigate a target based on initial Nmap scan results.
You have access to the following tools:
1. run_nikto(target) - Scan web server for vulnerabilities.
2. run_gobuster(target, wordlist) - Brute-force directories.
3. finish(summary) - End the session with a summary.

You must reply with a strictly valid JSON object representing your decision.
Example:
{ "action": "run_nikto", "args": { "target": "http://192.168.1.1" } }
or
{ "action": "finish", "args": { "summary": "Port 80 found, Nikto found nothing." } }

Do not output markdown code blocks. Just the raw JSON.
"#;

        let mut current_context = format!(
            "Target: {}
Initial Nmap Output:
{}
Wordlist: {}",
            target, initial_context, wordlist
        );
        let mut steps = 0;

        loop {
            if steps > 5 {
                println!("{}", "[ERROR] Max agent steps reached.".red());
                break;
            }
            steps += 1;

            println!("{}", format!("[AGENT] Thinking (Step {})...", steps).cyan());

            let response = self.client.chat(system_prompt, &current_context).await?;

            // Try to parse JSON
            // Clean markdown if present
            let clean_json = response
                .trim()
                .trim_start_matches("```json")
                .trim_end_matches("```");

            let action: AgentAction = match serde_json::from_str(clean_json) {
                Ok(a) => a,
                Err(e) => {
                    println!(
                        "{}",
                        format!("[ERROR] Failed to parse Agent response: {}", e).red()
                    );
                    println!("Response was: {}", response);
                    break;
                }
            };

            match action {
                AgentAction::RunNikto { target } => {
                    println!("{}", "[AGENT] Decided to run Nikto.".green());
                    let out = ToolRunner::run_nikto(&target, output_dir).await?;
                    current_context.push_str(&format!(
                        "
[Result of Nikto]
{}",
                        out
                    ));
                }
                AgentAction::RunGobuster { target, wordlist } => {
                    println!("{}", "[AGENT] Decided to run Gobuster.".green());
                    let out = ToolRunner::run_gobuster(&target, &wordlist, output_dir).await?;
                    current_context.push_str(&format!(
                        "
[Result of Gobuster]
{}",
                        out
                    ));
                }
                AgentAction::Finish { summary } => {
                    println!(
                        "{}",
                        "
[AGENT] Investigation Complete."
                            .blue()
                            .bold()
                    );
                    println!("{}", summary.white());
                    break;
                }
            }
        }

        Ok(())
    }
}
