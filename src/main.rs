mod cli;
mod gather;
mod report;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    let result = run(&cli);
    match result {
        Ok(()) => {}
        Err(e) => {
            let code = e.exit_code();
            if cli.is_json() {
                let err_json = serde_json::json!({
                    "ok": false,
                    "error": {
                        "code": e.error_code(),
                        "message": e.to_string(),
                    }
                });
                eprintln!("{}", serde_json::to_string_pretty(&err_json).unwrap());
            } else {
                eprintln!("error: {e}");
            }
            std::process::exit(code);
        }
    }
}

fn run(cli: &Cli) -> Result<(), StitchError> {
    let repo = cli.resolve_repo()?;

    match &cli.command {
        Command::Rebuild { depth, contents } => {
            let ctx = gather::gather_context(&repo, *depth, *contents)?;
            report::print_context(&ctx, cli.is_json())
        }
        Command::Sources => {
            let ctx = gather::gather_context(&repo, 1, false)?;
            report::print_sources(&ctx, cli.is_json())
        }
        Command::Brief => {
            let ctx = gather::gather_context(&repo, 2, true)?;
            let brief = gather::generate_brief(&ctx);
            report::print_brief(&brief, cli.is_json())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StitchError {
    #[error("{0}")]
    Validation(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl StitchError {
    pub fn exit_code(&self) -> i32 {
        match self {
            StitchError::Validation(_) => 1,
            StitchError::Io(_) => 2,
            StitchError::Json(_) => 1,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            StitchError::Validation(_) => "validation_error",
            StitchError::Io(_) => "io_error",
            StitchError::Json(_) => "json_error",
        }
    }
}
