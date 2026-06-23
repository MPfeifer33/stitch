use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::StitchError;

#[derive(Parser, Debug)]
#[command(name = "stitch", version, about = "Incremental context rebuilder for agents")]
pub struct Cli {
    /// Project root override
    #[arg(long, global = true)]
    pub repo: Option<PathBuf>,

    /// Output format
    #[arg(long, global = true, default_value = "text")]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn resolve_repo(&self) -> Result<PathBuf, StitchError> {
        if let Some(ref repo) = self.repo {
            return Ok(repo.clone());
        }
        if let Ok(output) = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(PathBuf::from(path));
            }
        }
        std::env::current_dir().map_err(StitchError::Io)
    }

    pub fn is_json(&self) -> bool {
        matches!(self.format, OutputFormat::Json)
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Rebuild context for the current project
    Rebuild {
        /// Max depth of files to include
        #[arg(long, default_value = "3")]
        depth: usize,
        /// Include file contents (not just structure)
        #[arg(long)]
        contents: bool,
    },
    /// Show what context sources are available
    Sources,
    /// Generate a compact project brief for cold-start
    Brief,
}
