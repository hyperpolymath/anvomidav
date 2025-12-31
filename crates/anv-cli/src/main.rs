// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Anvomidav CLI - Command-line interface for the figure skating DSL.
//!
//! This tool provides commands for:
//! - Checking Anvomidav source files for errors
//! - Formatting source files
//! - Running programs
//! - Generating visualizations

use anv_core::source::FileId;
use anv_syntax::parse;
use anv_types::check;
use clap::{Parser, Subcommand};
use miette::{Diagnostic, NamedSource, Report, SourceSpan};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

/// Anvomidav - A domain-specific language for figure skating choreography.
#[derive(Parser)]
#[command(name = "anv")]
#[command(author = "hyperpolymath")]
#[command(version)]
#[command(about = "Anvomidav figure skating DSL compiler and tools", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check source files for errors without running them
    Check {
        /// Source files to check
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Show detailed output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Parse a source file and print the AST
    Parse {
        /// Source file to parse
        file: PathBuf,

        /// Output format (json, debug)
        #[arg(short, long, default_value = "debug")]
        format: String,
    },

    /// Lex a source file and print tokens
    Lex {
        /// Source file to lex
        file: PathBuf,
    },

    /// Format source files
    Fmt {
        /// Source files to format
        files: Vec<PathBuf>,

        /// Check if files are formatted without modifying them
        #[arg(long)]
        check: bool,
    },

    /// Create a new Anvomidav project
    New {
        /// Project name
        name: String,

        /// Project template (singles, pairs, ice-dance)
        #[arg(short, long, default_value = "singles")]
        template: String,
    },
}

/// Error type for CLI operations.
#[derive(Debug, thiserror::Error, Diagnostic)]
#[allow(clippy::enum_variant_names)]
enum CliError {
    #[error("failed to read file: {path}")]
    #[diagnostic(code(anv::io::read_error))]
    ReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("parse error")]
    #[diagnostic(code(anv::syntax::parse_error))]
    ParseError {
        #[source_code]
        src: NamedSource<String>,
        #[label("here")]
        span: SourceSpan,
        #[help]
        help: String,
    },

    #[error("type error")]
    #[diagnostic(code(anv::types::type_error))]
    #[allow(dead_code)]
    TypeError {
        #[source_code]
        src: NamedSource<String>,
        #[label("{message}")]
        span: SourceSpan,
        message: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{:?}", e);
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> miette::Result<()> {
    match cli.command {
        Commands::Check { files, verbose } => {
            let mut has_errors = false;

            for (file_idx, path) in files.iter().enumerate() {
                if verbose {
                    eprintln!("Checking {}...", path.display());
                }

                let source = fs::read_to_string(path).map_err(|e| CliError::ReadError {
                    path: path.display().to_string(),
                    source: e,
                })?;

                let file_id = FileId(file_idx as u32);

                // Parse
                let program = match parse(&source, file_id) {
                    Ok(p) => p,
                    Err(errors) => {
                        has_errors = true;
                        for err in errors {
                            let span: SourceSpan = (err.span.start, err.span.end - err.span.start).into();
                            let report = Report::new(CliError::ParseError {
                                src: NamedSource::new(path.display().to_string(), source.clone()),
                                span,
                                help: err.message.clone(),
                            });
                            eprintln!("{:?}", report);
                        }
                        continue;
                    }
                };

                // Type check
                if let Err(diagnostics) = check(&program, file_id) {
                    has_errors = true;
                    for diag in diagnostics.iter() {
                        let code_str = diag.code.as_ref().map(|c| c.to_string()).unwrap_or_default();
                        eprintln!(
                            "{}:{}: {} [{}]",
                            path.display(),
                            diag.labels.first().map(|l| l.span.start).unwrap_or(0),
                            diag.message,
                            code_str
                        );
                        for note in &diag.notes {
                            eprintln!("  note: {:?}", note);
                        }
                    }
                }
            }

            if has_errors {
                Err(miette::miette!("check failed with errors"))?;
            } else if verbose {
                eprintln!("All checks passed!");
            }

            Ok(())
        }

        Commands::Parse { file, format } => {
            let source = fs::read_to_string(&file).map_err(|e| CliError::ReadError {
                path: file.display().to_string(),
                source: e,
            })?;

            let program = parse(&source, FileId(0)).map_err(|errors| {
                let err = &errors[0];
                let span: SourceSpan = (err.span.start, err.span.end - err.span.start).into();
                CliError::ParseError {
                    src: NamedSource::new(file.display().to_string(), source.clone()),
                    span,
                    help: err.message.clone(),
                }
            })?;

            match format.as_str() {
                "debug" => println!("{:#?}", program),
                "json" => {
                    // TODO: Implement JSON serialization
                    println!("{{\"name\": \"{}\", \"segments\": {}}}", program.name.node, program.segments.len());
                }
                _ => {
                    eprintln!("Unknown format: {}. Using 'debug'.", format);
                    println!("{:#?}", program);
                }
            }

            Ok(())
        }

        Commands::Lex { file } => {
            let source = fs::read_to_string(&file).map_err(|e| CliError::ReadError {
                path: file.display().to_string(),
                source: e,
            })?;

            let tokens = anv_syntax::Lexer::tokenize(&source).map_err(|e| {
                let span: SourceSpan = (e.span.start, e.span.end - e.span.start).into();
                CliError::ParseError {
                    src: NamedSource::new(file.display().to_string(), source.clone()),
                    span,
                    help: e.message.clone(),
                }
            })?;

            for tok in tokens {
                println!("{:4}..{:4} {:?}", tok.span.start, tok.span.end, tok.token);
            }

            Ok(())
        }

        Commands::Fmt { files, check } => {
            if files.is_empty() {
                eprintln!("No files specified");
                return Ok(());
            }

            for path in &files {
                let source = fs::read_to_string(path).map_err(|e| CliError::ReadError {
                    path: path.display().to_string(),
                    source: e,
                })?;

                // TODO: Implement formatter
                // For now, just verify the file parses
                let _program = parse(&source, FileId(0)).map_err(|errors| {
                    let err = &errors[0];
                    let span: SourceSpan = (err.span.start, err.span.end - err.span.start).into();
                    CliError::ParseError {
                        src: NamedSource::new(path.display().to_string(), source.clone()),
                        span,
                        help: err.message.clone(),
                    }
                })?;

                if check {
                    eprintln!("{}: would reformat", path.display());
                } else {
                    eprintln!("{}: formatted (no-op, formatter not implemented)", path.display());
                }
            }

            Ok(())
        }

        Commands::New { name, template } => {
            let project_dir = PathBuf::from(&name);

            if project_dir.exists() {
                return Err(miette::miette!("directory '{}' already exists", name));
            }

            fs::create_dir_all(&project_dir).map_err(|e| CliError::ReadError {
                path: project_dir.display().to_string(),
                source: e,
            })?;

            let main_content = match template.as_str() {
                "singles" => format!(
                    r#"/// {} - Singles figure skating program
///
/// Created with Anvomidav

program {} {{
    segment short_program: short {{
        sequence opening {{
            jump triple axel at 0:15
            spin camel sit upright L3 at 0:30
            step circular L3 at 1:00
        }}
    }}

    segment free_skate: free {{
        sequence technical {{
            jump quad lutz at 0:30
            jump triple axel at 1:00
            spin layback L4 at 1:30
        }}
    }}
}}
"#,
                    name, name
                ),
                "pairs" => format!(
                    r#"/// {} - Pairs figure skating program
///
/// Created with Anvomidav

program {} {{
    segment short_program: short {{
        skater lead
        skater follow

        sequence opening {{
            parallel {{
                lead: jump double axel at 0:15
                follow: jump double axel at 0:15
            }}
            lift group3 L3 at 0:30
            throw triple salchow at 1:00
        }}
    }}
}}
"#,
                    name, name
                ),
                "ice-dance" => format!(
                    r#"/// {} - Ice dance program
///
/// Created with Anvomidav

program {} {{
    segment pattern: pattern {{
        skater lead
        skater follow

        sequence waltz {{
            sync {{
                step circular L3 at 0:00
            }}
        }}
    }}
}}
"#,
                    name, name
                ),
                _ => {
                    return Err(miette::miette!(
                        "unknown template '{}'. Use: singles, pairs, or ice-dance",
                        template
                    ));
                }
            };

            let main_file = project_dir.join("main.anv");
            fs::write(&main_file, main_content).map_err(|e| CliError::ReadError {
                path: main_file.display().to_string(),
                source: e,
            })?;

            eprintln!("Created new {} project in '{}'", template, name);
            eprintln!("  Main file: {}/main.anv", name);

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse() {
        let cli = Cli::try_parse_from(["anv", "check", "test.anv"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_cli_version() {
        let cli = Cli::try_parse_from(["anv", "--version"]);
        // --version causes early exit, so this will be an error
        assert!(cli.is_err());
    }
}
