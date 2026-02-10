use anyhow::{Context, Result};
use colored::*;
use std::path::Path;
use std::process::Command;

pub struct ToolRunner;

impl ToolRunner {
    pub async fn run_nmap(target: &str, output_dir: &Path) -> Result<String> {
        println!("{}", format!("[SYSTEM] Running Nmap on {}", target).green());
        let output_file = output_dir.join("nmap.txt");

        let mut cmd = Command::new("nmap");
        cmd.arg("-sV")
            .arg("-sC")
            .arg("-oN")
            .arg(&output_file)
            .arg(target);

        let output = cmd.output().context("Failed to execute nmap")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Nmap failed: {}", stderr));
        }

        Ok(stdout)
    }

    pub async fn run_nikto(target: &str, output_dir: &Path) -> Result<String> {
        println!(
            "{}",
            format!("[SYSTEM] Running Nikto on {}", target).green()
        );
        let output_file = output_dir.join("nikto.txt");

        let mut cmd = Command::new("nikto");
        cmd.arg("-h").arg(target).arg("-o").arg(&output_file);

        let output = cmd.output().context("Failed to execute nikto")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        // Nikto might exit with non-zero on some findings, so we just check if it ran.
        Ok(stdout)
    }

    pub async fn run_gobuster(target: &str, wordlist: &str, output_dir: &Path) -> Result<String> {
        println!(
            "{}",
            format!("[SYSTEM] Running Gobuster on {}", target).green()
        );
        let output_file = output_dir.join("gobuster.txt");

        let mut cmd = Command::new("gobuster");
        cmd.arg("dir")
            .arg("-u")
            .arg(target)
            .arg("-w")
            .arg(wordlist)
            .arg("-o")
            .arg(&output_file)
            .arg("-k"); // Skip SSL verification

        let output = cmd.output().context("Failed to execute gobuster")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }
}
