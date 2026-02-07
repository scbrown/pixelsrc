//! Agent and LSP command implementations

use clap::Subcommand;
use std::path::PathBuf;
use std::process::ExitCode;

use super::{EXIT_ERROR, EXIT_INVALID_ARGS, EXIT_SUCCESS};

#[derive(Subcommand)]
pub enum AgentAction {
    /// Verify content and return structured diagnostics
    Verify {
        /// Input file to verify (omit for stdin)
        file: Option<PathBuf>,

        /// Read from stdin
        #[arg(long)]
        stdin: bool,

        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
    },
    /// Get token completions at a position
    Completions {
        /// Input file
        file: Option<PathBuf>,

        /// Read from stdin
        #[arg(long)]
        stdin: bool,

        /// Line number (1-indexed)
        #[arg(long, default_value = "1")]
        line: usize,

        /// Character position (0-indexed)
        #[arg(long, default_value = "0")]
        character: usize,
    },
    /// Get grid position information
    Position {
        /// Input file
        file: Option<PathBuf>,

        /// Read from stdin
        #[arg(long)]
        stdin: bool,

        /// Line number (1-indexed)
        #[arg(long)]
        line: usize,

        /// Character position (0-indexed)
        #[arg(long)]
        character: usize,
    },
}

/// Execute the LSP server command
#[cfg(feature = "lsp")]
pub fn run_lsp() -> ExitCode {
    use tokio::runtime::Runtime;

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Error: Failed to create async runtime: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    rt.block_on(crate::lsp::run_server());
    ExitCode::from(EXIT_SUCCESS)
}

/// Execute agent command (verify, completions, position)
pub fn run_agent(action: AgentAction) -> ExitCode {
    use crate::lsp_agent_client::LspAgentClient;
    use std::io::{self, Read};

    match action {
        AgentAction::Verify { file, stdin, strict } => {
            let content = if stdin {
                let mut buf = String::new();
                if let Err(e) = io::stdin().read_to_string(&mut buf) {
                    eprintln!("Error reading stdin: {}", e);
                    return ExitCode::from(EXIT_ERROR);
                }
                buf
            } else if let Some(path) = file {
                match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error reading file: {}", e);
                        return ExitCode::from(EXIT_ERROR);
                    }
                }
            } else {
                eprintln!("Error: Provide a file or use --stdin");
                return ExitCode::from(EXIT_INVALID_ARGS);
            };

            let client = if strict { LspAgentClient::strict() } else { LspAgentClient::new() };
            println!("{}", client.verify_content_json(&content));
            ExitCode::from(EXIT_SUCCESS)
        }
        AgentAction::Completions { file, stdin, line, character } => {
            let content = if stdin {
                let mut buf = String::new();
                if let Err(e) = io::stdin().read_to_string(&mut buf) {
                    eprintln!("Error reading stdin: {}", e);
                    return ExitCode::from(EXIT_ERROR);
                }
                buf
            } else if let Some(path) = file {
                match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error reading file: {}", e);
                        return ExitCode::from(EXIT_ERROR);
                    }
                }
            } else {
                eprintln!("Error: Provide a file or use --stdin");
                return ExitCode::from(EXIT_INVALID_ARGS);
            };

            let client = LspAgentClient::new();
            println!("{}", client.get_completions_json(&content, line, character));
            ExitCode::from(EXIT_SUCCESS)
        }
        AgentAction::Position { file, stdin, line, character } => {
            let content = if stdin {
                let mut buf = String::new();
                if let Err(e) = io::stdin().read_to_string(&mut buf) {
                    eprintln!("Error reading stdin: {}", e);
                    return ExitCode::from(EXIT_ERROR);
                }
                buf
            } else if let Some(path) = file {
                match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error reading file: {}", e);
                        return ExitCode::from(EXIT_ERROR);
                    }
                }
            } else {
                eprintln!("Error: Provide a file or use --stdin");
                return ExitCode::from(EXIT_INVALID_ARGS);
            };

            let client = LspAgentClient::new();
            if let Some(pos) = client.get_grid_position(&content, line, character) {
                println!("{}", serde_json::to_string_pretty(&pos).expect("JSON value serialization"));
            } else {
                println!("null");
            }
            ExitCode::from(EXIT_SUCCESS)
        }
    }
}
