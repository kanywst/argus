use anyhow::Result;
use clap::Parser;
use colored::*;
use dotenvy::dotenv;
use std::path::{Path, PathBuf};

mod agent;
mod llm;
mod tools;

#[derive(Parser, Debug)]
#[command(name = "argus")]
#[command(about = "Modern AI-Powered Network Reconnaissance Agent", long_about = None)]
#[command(version)]
struct Args {
    /// Target URL or IP
    #[arg(required = true)]
    target: String,

    /// Path to wordlist for gobuster
    #[arg(short, long, default_value = "/usr/share/dirb/wordlists/big.txt")]
    wordlist: String,

    /// Output directory
    #[arg(short, long, default_value = "results")]
    output: PathBuf,

    /// Enable AI Agent mode (requires OPENAI_API_KEY or OLLAMA_HOST)
    #[arg(long, default_value_t = false)]
    ai: bool,

    /// AI Provider (openai, ollama)
    #[arg(long, default_value = "openai")]
    provider: String,

    /// Model name (e.g., gpt-4o, llama3)
    #[arg(long)]
    model: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let args = Args::parse();

    println!(
        "{}",
        r#"
    ___    ____  ______ __  __ _____
   /   |  / __ \/ ____// / / // ___/
  / /| | / /_/ / / __ / / / / \__ \ 
 / ___ |/ _, _/ /_/ // /_/ / ___/ / 
/_/  |_/_/ |_|\____/ \____/ /____/  
    "#
        .white()
        .bold()
    );
    println!("SYSTEM: AUTONOMOUS RECONNAISSANCE ENGINE");
    println!("TARGET: {}", args.target.white());
    println!("AGENT:  {}", if args.ai { "ACTIVE" } else { "DISABLED" });

    // Ensure output directory exists
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output)?;
    }

    // 1. Initial Recon (Nmap) - Always run this first as context
    let nmap_output = tools::ToolRunner::run_nmap(&args.target, &args.output).await?;

    // 2. Decide next steps
    if args.ai {
        println!("{}", "[*] AI Agent Analyzing results...".magenta());
        let agent = agent::Agent::new(args.provider, args.model);
        agent
            .run_loop(&args.target, &args.wordlist, &args.output, &nmap_output)
            .await?;
    } else {
        // Classic Logic
        classic_mode(&args.target, &args.wordlist, &args.output, &nmap_output).await?;
    }

    println!("\n{}", "[OK] Scan Complete.".green().bold());
    Ok(())
}

async fn classic_mode(target: &str, wordlist: &str, output: &Path, nmap_out: &str) -> Result<()> {
    // Simple heuristic parsing
    if nmap_out.contains("80/tcp") || nmap_out.contains("443/tcp") || nmap_out.contains("http") {
        println!(
            "{}",
            "[*] Web ports detected. Starting Web Recon...".yellow()
        );

        let _ = tools::ToolRunner::run_nikto(target, output).await?;
        let _ = tools::ToolRunner::run_gobuster(target, wordlist, output).await?;
    }

    // Add more heuristics here (e.g. SMB, SSH)
    Ok(())
}
