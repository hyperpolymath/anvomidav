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
use anv_ir::lower;
use anv_semantics::{validate_program, Discipline};
use anv_syntax::parse;
use anv_types::check;
use anv_viz::{RinkRenderer, SvgOptions, TimelineRenderer};
use clap::{Parser, Subcommand};
use miette::{Diagnostic, NamedSource, Report, SourceSpan};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::mpsc::channel;
use std::time::Duration;

/// Anvomidav - A domain-specific language for figure skating choreography.
#[derive(Parser)]
#[command(name = "anv")]
#[command(author = "hyperpolymath")]
#[command(version)]
#[command(about = "Anvomidav figure skating DSL compiler and tools")]
#[command(long_about = "Anvomidav is a domain-specific language for describing figure skating \
programs. It supports singles, pairs, and ice dance disciplines with \
ISU rule validation.

EXAMPLES:
    anv check program.anv           Check a program for errors
    anv viz program.anv             Generate rink visualization
    anv export program.anv --pretty Export to JSON format
    anv new my_program --template pairs  Create new pairs project")]
#[command(after_help = "See 'anv <command> --help' for more information on a specific command.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check source files for errors without running them
    #[command(long_about = "Check one or more Anvomidav source files for syntax and type errors.

EXAMPLES:
    anv check program.anv
    anv check *.anv --verbose
    anv check short.anv free.anv
    anv check *.anv --watch       (watch for changes)")]
    Check {
        /// Source files to check
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Show detailed output
        #[arg(short, long)]
        verbose: bool,

        /// Watch for file changes and re-check automatically
        #[arg(short, long)]
        watch: bool,
    },

    /// Parse a source file and print the AST
    #[command(long_about = "Parse a source file and display the abstract syntax tree.

EXAMPLES:
    anv parse program.anv
    anv parse program.anv --format json > ast.json")]
    Parse {
        /// Source file to parse
        file: PathBuf,

        /// Output format (json, debug)
        #[arg(short, long, default_value = "debug")]
        format: String,
    },

    /// Lex a source file and print tokens
    #[command(long_about = "Tokenize a source file and display the token stream.
Useful for debugging lexer issues.

EXAMPLES:
    anv lex program.anv")]
    Lex {
        /// Source file to lex
        file: PathBuf,
    },

    /// Format source files
    #[command(long_about = "Format Anvomidav source files to canonical style.

EXAMPLES:
    anv fmt program.anv
    anv fmt *.anv --check     (verify formatting without changes)
    anv fmt *.anv --watch     (watch for changes)")]
    Fmt {
        /// Source files to format
        files: Vec<PathBuf>,

        /// Check if files are formatted without modifying them
        #[arg(long)]
        check: bool,

        /// Watch for file changes and re-format automatically
        #[arg(short, long)]
        watch: bool,
    },

    /// Create a new Anvomidav project
    #[command(long_about = "Create a new Anvomidav project with template files.

TEMPLATES:
    singles     Singles skating (men's or ladies')
    pairs       Pairs skating with lifts, throws, etc.
    ice-dance   Ice dance with patterns and rhythm

EXAMPLES:
    anv new my_program
    anv new competition_sp --template pairs")]
    New {
        /// Project name
        name: String,

        /// Project template (singles, pairs, ice-dance)
        #[arg(short, long, default_value = "singles")]
        template: String,
    },

    /// Generate SVG visualization of a program
    #[command(long_about = "Generate SVG visualizations of skating programs.

VISUALIZATION TYPES:
    rink        Ice rink diagram with element positions
    timeline    Timeline chart showing element sequence
    both        Generate both rink and timeline SVGs

EXAMPLES:
    anv viz program.anv
    anv viz program.anv -o output.svg
    anv viz program.anv --viz-type timeline
    anv viz program.anv --viz-type both --width 1200")]
    Viz {
        /// Source file to visualize
        file: PathBuf,

        /// Output file (default: <input>.svg)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Visualization type (rink, timeline, both)
        #[arg(short = 't', long, default_value = "rink")]
        viz_type: String,

        /// SVG width in pixels
        #[arg(long, default_value = "800")]
        width: f64,

        /// SVG height in pixels
        #[arg(long, default_value = "400")]
        height: f64,
    },

    /// Export program to different formats
    #[command(long_about = "Export skating programs to various formats.

FORMATS:
    json    Timeline data as JSON (for external tools)
    ir      Internal representation (for debugging)

EXAMPLES:
    anv export program.anv
    anv export program.anv --pretty -o data.json")]
    Export {
        /// Source file to export
        file: PathBuf,

        /// Output file (default: <input>.<format>)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Export format (json, ir)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Pretty-print output
        #[arg(long)]
        pretty: bool,
    },

    /// Display information about a program
    #[command(long_about = "Display detailed information about a skating program.

Shows element counts, segment details, and optionally validates
against ISU (International Skating Union) rules.

DISCIPLINES:
    singles     Men's or Ladies' singles skating
    ladies      Ladies' singles (alias)
    pairs       Pairs skating
    ice-dance   Ice dance

EXAMPLES:
    anv info program.anv
    anv info program.anv --validate
    anv info pairs.anv --validate --discipline pairs")]
    Info {
        /// Source file to analyze
        file: PathBuf,

        /// Show ISU rule validation
        #[arg(long)]
        validate: bool,

        /// Discipline for validation (singles, pairs, ice-dance)
        #[arg(short, long, default_value = "singles")]
        discipline: String,
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

    #[error("{message}")]
    #[diagnostic(code(anv::syntax::parse_error))]
    ParseError {
        #[source_code]
        src: NamedSource<String>,
        #[label("{label}")]
        span: SourceSpan,
        message: String,
        label: String,
        #[help]
        help: Option<String>,
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

/// Check files for errors, returning true if all passed.
fn check_files(files: &[PathBuf], verbose: bool) -> bool {
    let mut has_errors = false;

    for (file_idx, path) in files.iter().enumerate() {
        if verbose {
            eprintln!("Checking {}...", path.display());
        }

        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to read {}: {}", path.display(), e);
                has_errors = true;
                continue;
            }
        };

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
                        message: err.message.clone(),
                        label: err.label.clone().unwrap_or_else(|| "here".into()),
                        help: err.help.clone(),
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

    if !has_errors && verbose {
        eprintln!("All checks passed!");
    }

    !has_errors
}

/// Format files, returning true if all formatted successfully.
fn format_files(files: &[PathBuf], check_only: bool) -> bool {
    if files.is_empty() {
        eprintln!("No files specified");
        return true;
    }

    let mut success = true;

    for path in files {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to read {}: {}", path.display(), e);
                success = false;
                continue;
            }
        };

        // Format the source
        let formatted = match anv_syntax::format::format(&source) {
            Ok(f) => f,
            Err(errors) => {
                for err in errors {
                    let span: SourceSpan = (err.span.start, err.span.end - err.span.start).into();
                    let report = Report::new(CliError::ParseError {
                        src: NamedSource::new(path.display().to_string(), source.clone()),
                        span,
                        message: err.message.clone(),
                        label: err.label.clone().unwrap_or_else(|| "here".into()),
                        help: err.help.clone(),
                    });
                    eprintln!("{:?}", report);
                }
                success = false;
                continue;
            }
        };

        let changed = source != formatted;

        if check_only {
            if changed {
                eprintln!("{}: would be reformatted", path.display());
                success = false;
            } else {
                eprintln!("{}: ok", path.display());
            }
        } else if changed {
            if let Err(e) = fs::write(path, &formatted) {
                eprintln!("Failed to write {}: {}", path.display(), e);
                success = false;
            } else {
                eprintln!("{}: formatted", path.display());
            }
        } else {
            eprintln!("{}: unchanged", path.display());
        }
    }

    success
}

/// Watch files for changes and run a callback when they change.
fn watch_files<F>(files: &[PathBuf], mut on_change: F) -> miette::Result<()>
where
    F: FnMut() -> bool,
{
    eprintln!("Watching for changes... (press Ctrl+C to stop)");

    // Run once initially
    on_change();

    // Set up file watcher
    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_millis(300), tx)
        .map_err(|e| miette::miette!("Failed to create watcher: {}", e))?;

    // Watch parent directories of all files
    let mut watched_dirs = std::collections::HashSet::new();
    for path in files {
        if let Some(parent) = path.parent() {
            let parent = if parent.as_os_str().is_empty() {
                PathBuf::from(".")
            } else {
                parent.to_path_buf()
            };
            if watched_dirs.insert(parent.clone()) {
                debouncer
                    .watcher()
                    .watch(&parent, RecursiveMode::NonRecursive)
                    .map_err(|e| miette::miette!("Failed to watch {}: {}", parent.display(), e))?;
            }
        }
    }

    // Process events
    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                // Check if any of our files changed
                let our_files: std::collections::HashSet<_> = files
                    .iter()
                    .filter_map(|p| p.canonicalize().ok())
                    .collect();

                let changed = events.iter().any(|event| {
                    event
                        .path
                        .canonicalize()
                        .map(|p| our_files.contains(&p))
                        .unwrap_or(false)
                });

                if changed {
                    eprintln!("\n--- File changed, re-running... ---\n");
                    on_change();
                }
            }
            Ok(Err(error)) => {
                eprintln!("Watch error: {}", error);
            }
            Err(e) => {
                eprintln!("Watch channel error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn run(cli: Cli) -> miette::Result<()> {
    match cli.command {
        Commands::Check { files, verbose, watch } => {
            if watch {
                watch_files(&files, || check_files(&files, verbose))?;
            } else {
                let success = check_files(&files, verbose);
                if !success {
                    return Err(miette::miette!("check failed with errors"));
                }
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
                    message: err.message.clone(),
                    label: err.label.clone().unwrap_or_else(|| "here".into()),
                    help: err.help.clone(),
                }
            })?;

            match format.as_str() {
                "debug" => println!("{:#?}", program),
                "json" => {
                    // Serialize program to JSON with essential structure
                    let segments: Vec<serde_json::Value> = program
                        .segments
                        .iter()
                        .map(|seg| {
                            let sequences: Vec<serde_json::Value> = seg
                                .sequences
                                .iter()
                                .map(|seq| {
                                    let elements: Vec<serde_json::Value> = seq
                                        .elements
                                        .iter()
                                        .map(|elem| {
                                            let kind = match &elem.kind {
                                                anv_syntax::ast::ElementKind::Jump(j) => {
                                                    serde_json::json!({
                                                        "type": "jump",
                                                        "kind": format!("{:?}", j.kind),
                                                        "rotations": format!("{:?}", j.rotations)
                                                    })
                                                }
                                                anv_syntax::ast::ElementKind::Spin(s) => {
                                                    serde_json::json!({
                                                        "type": "spin",
                                                        "positions": s.positions.iter()
                                                            .map(|p| format!("{:?}", p.position))
                                                            .collect::<Vec<_>>(),
                                                        "level": s.level.map(|l| format!("{}", l))
                                                    })
                                                }
                                                anv_syntax::ast::ElementKind::StepSequence(s) => {
                                                    serde_json::json!({
                                                        "type": "step_sequence",
                                                        "pattern": format!("{}", s.pattern),
                                                        "level": s.level.map(|l| format!("{}", l))
                                                    })
                                                }
                                                anv_syntax::ast::ElementKind::Lift(l) => {
                                                    serde_json::json!({
                                                        "type": "lift",
                                                        "group": format!("{}", l.group),
                                                        "level": l.level.map(|l| format!("{}", l))
                                                    })
                                                }
                                                anv_syntax::ast::ElementKind::Throw(t) => {
                                                    serde_json::json!({
                                                        "type": "throw",
                                                        "kind": format!("{:?}", t.kind),
                                                        "rotations": format!("{:?}", t.rotations)
                                                    })
                                                }
                                                anv_syntax::ast::ElementKind::Twist(t) => {
                                                    serde_json::json!({
                                                        "type": "twist",
                                                        "rotations": format!("{:?}", t.rotations),
                                                        "level": t.level.map(|l| format!("{}", l))
                                                    })
                                                }
                                                anv_syntax::ast::ElementKind::DeathSpiral(d) => {
                                                    serde_json::json!({
                                                        "type": "death_spiral",
                                                        "edge": format!("{}", d.edge),
                                                        "level": d.level.map(|l| format!("{}", l))
                                                    })
                                                }
                                                anv_syntax::ast::ElementKind::Choreographic(c) => {
                                                    serde_json::json!({
                                                        "type": "choreographic",
                                                        "kind": format!("{:?}", c.kind)
                                                    })
                                                }
                                                anv_syntax::ast::ElementKind::Pattern(p) => {
                                                    serde_json::json!({
                                                        "type": "pattern",
                                                        "name": p.name.clone()
                                                    })
                                                }
                                                anv_syntax::ast::ElementKind::Transition(_) => {
                                                    serde_json::json!({"type": "transition"})
                                                }
                                                anv_syntax::ast::ElementKind::Parallel(_) => {
                                                    serde_json::json!({"type": "parallel"})
                                                }
                                                anv_syntax::ast::ElementKind::Sync(_) => {
                                                    serde_json::json!({"type": "sync"})
                                                }
                                            };
                                            kind
                                        })
                                        .collect();
                                    serde_json::json!({
                                        "name": seq.name.as_ref().map(|n| n.node.clone()),
                                        "elements": elements
                                    })
                                })
                                .collect();
                            serde_json::json!({
                                "name": seg.name.node.clone(),
                                "kind": format!("{}", seg.kind),
                                "sequences": sequences
                            })
                        })
                        .collect();

                    let output = serde_json::json!({
                        "name": program.name.node,
                        "segments": segments,
                        "functions": program.functions.len(),
                        "types": program.types.len(),
                        "imports": program.imports.len()
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
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
                    message: format!("lexer error: {}", e.message),
                    label: "here".into(),
                    help: Some("check for invalid characters or unclosed strings".into()),
                }
            })?;

            for tok in tokens {
                println!("{:4}..{:4} {:?}", tok.span.start, tok.span.end, tok.token);
            }

            Ok(())
        }

        Commands::Fmt { files, check: check_only, watch } => {
            if watch {
                watch_files(&files, || format_files(&files, check_only))?;
            } else {
                let success = format_files(&files, check_only);
                if !success && check_only {
                    return Err(miette::miette!("Some files would be reformatted"));
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

        Commands::Viz {
            file,
            output,
            viz_type,
            width,
            height,
        } => {
            let source = fs::read_to_string(&file).map_err(|e| CliError::ReadError {
                path: file.display().to_string(),
                source: e,
            })?;

            // Parse and type check
            let program = parse(&source, FileId(0)).map_err(|errors| {
                let err = &errors[0];
                let span: SourceSpan = (err.span.start, err.span.end - err.span.start).into();
                CliError::ParseError {
                    src: NamedSource::new(file.display().to_string(), source.clone()),
                    span,
                    message: err.message.clone(),
                    label: err.label.clone().unwrap_or_else(|| "here".into()),
                    help: err.help.clone(),
                }
            })?;

            check(&program, FileId(0)).map_err(|diagnostics| {
                miette::miette!(
                    "Type errors: {}",
                    diagnostics
                        .iter()
                        .map(|d| d.message.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;

            // Lower to IR
            let timeline = lower(&program);

            // Generate SVG based on type
            let base_output = output.unwrap_or_else(|| {
                let mut p = file.clone();
                p.set_extension("svg");
                p
            });

            match viz_type.as_str() {
                "rink" => {
                    let options = SvgOptions {
                        width,
                        height,
                        ..Default::default()
                    };
                    let svg = RinkRenderer::new(options).render(&timeline);
                    fs::write(&base_output, &svg).map_err(|e| CliError::ReadError {
                        path: base_output.display().to_string(),
                        source: e,
                    })?;
                    eprintln!("Generated rink diagram: {}", base_output.display());
                }
                "timeline" => {
                    let renderer = TimelineRenderer::default_renderer();
                    let svg = renderer.render(&timeline);
                    fs::write(&base_output, &svg).map_err(|e| CliError::ReadError {
                        path: base_output.display().to_string(),
                        source: e,
                    })?;
                    eprintln!("Generated timeline chart: {}", base_output.display());
                }
                "both" => {
                    // Rink diagram
                    let rink_options = SvgOptions {
                        width,
                        height,
                        ..Default::default()
                    };
                    let rink_svg = RinkRenderer::new(rink_options).render(&timeline);
                    let mut rink_path = base_output.clone();
                    rink_path.set_extension("rink.svg");
                    fs::write(&rink_path, &rink_svg).map_err(|e| CliError::ReadError {
                        path: rink_path.display().to_string(),
                        source: e,
                    })?;
                    eprintln!("Generated rink diagram: {}", rink_path.display());

                    // Timeline chart
                    let timeline_svg = TimelineRenderer::default_renderer().render(&timeline);
                    let mut timeline_path = base_output.clone();
                    timeline_path.set_extension("timeline.svg");
                    fs::write(&timeline_path, &timeline_svg).map_err(|e| CliError::ReadError {
                        path: timeline_path.display().to_string(),
                        source: e,
                    })?;
                    eprintln!("Generated timeline chart: {}", timeline_path.display());
                }
                _ => {
                    return Err(miette::miette!(
                        "Unknown viz type '{}'. Use: rink, timeline, or both",
                        viz_type
                    ));
                }
            }

            Ok(())
        }

        Commands::Export {
            file,
            output,
            format,
            pretty,
        } => {
            let source = fs::read_to_string(&file).map_err(|e| CliError::ReadError {
                path: file.display().to_string(),
                source: e,
            })?;

            // Parse and type check
            let program = parse(&source, FileId(0)).map_err(|errors| {
                let err = &errors[0];
                let span: SourceSpan = (err.span.start, err.span.end - err.span.start).into();
                CliError::ParseError {
                    src: NamedSource::new(file.display().to_string(), source.clone()),
                    span,
                    message: err.message.clone(),
                    label: err.label.clone().unwrap_or_else(|| "here".into()),
                    help: err.help.clone(),
                }
            })?;

            check(&program, FileId(0)).map_err(|diagnostics| {
                miette::miette!(
                    "Type errors: {}",
                    diagnostics
                        .iter()
                        .map(|d| d.message.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;

            // Lower to IR
            let timeline = lower(&program);

            let ext = match format.as_str() {
                "json" => "json",
                "ir" => "ir.json",
                _ => {
                    return Err(miette::miette!(
                        "Unknown export format '{}'. Use: json or ir",
                        format
                    ));
                }
            };

            let output_path = output.unwrap_or_else(|| {
                let mut p = file.clone();
                p.set_extension(ext);
                p
            });

            let json = if pretty {
                serde_json::to_string_pretty(&timeline)
            } else {
                serde_json::to_string(&timeline)
            }
            .map_err(|e| miette::miette!("JSON serialization failed: {}", e))?;

            fs::write(&output_path, &json).map_err(|e| CliError::ReadError {
                path: output_path.display().to_string(),
                source: e,
            })?;

            eprintln!("Exported to: {}", output_path.display());
            Ok(())
        }

        Commands::Info { file, validate, discipline } => {
            let source = fs::read_to_string(&file).map_err(|e| CliError::ReadError {
                path: file.display().to_string(),
                source: e,
            })?;

            // Parse
            let program = parse(&source, FileId(0)).map_err(|errors| {
                let err = &errors[0];
                let span: SourceSpan = (err.span.start, err.span.end - err.span.start).into();
                CliError::ParseError {
                    src: NamedSource::new(file.display().to_string(), source.clone()),
                    span,
                    message: err.message.clone(),
                    label: err.label.clone().unwrap_or_else(|| "here".into()),
                    help: err.help.clone(),
                }
            })?;

            // Type check
            let type_ok = check(&program, FileId(0)).is_ok();

            // Lower to IR
            let timeline = lower(&program);

            println!("Program: {}", program.name.node);
            println!("Segments: {}", program.segments.len());
            println!("Type Check: {}", if type_ok { "PASS" } else { "FAIL" });
            println!();

            // Count elements by type
            let mut jumps = 0;
            let mut spins = 0;
            let mut steps = 0;
            let mut choreo = 0;
            let mut pairs_elements = 0;

            for event in &timeline.events {
                use anv_ir::timeline::EventKind;
                match &event.kind {
                    EventKind::Jump { .. } | EventKind::JumpCombination { .. } => jumps += 1,
                    EventKind::Spin { .. } => spins += 1,
                    EventKind::StepSequence { .. } => steps += 1,
                    EventKind::ChoreographicSequence | EventKind::Choreographic { .. } => choreo += 1,
                    EventKind::Lift { .. }
                    | EventKind::Throw { .. }
                    | EventKind::Twist { .. }
                    | EventKind::DeathSpiral { .. } => pairs_elements += 1,
                    _ => {}
                }
            }

            println!("Elements:");
            println!("  Jumps: {}", jumps);
            println!("  Spins: {}", spins);
            println!("  Steps: {}", steps);
            println!("  Choreographic: {}", choreo);
            if pairs_elements > 0 {
                println!("  Pairs Elements: {}", pairs_elements);
            }
            println!("  Total: {}", timeline.events.len());
            println!();

            if validate {
                let disc = match discipline.as_str() {
                    "singles" | "men" => Discipline::MenSingles,
                    "ladies" | "women" => Discipline::LadiesSingles,
                    "pairs" => Discipline::Pairs,
                    "ice-dance" | "dance" => Discipline::IceDance,
                    _ => {
                        return Err(miette::miette!(
                            "Unknown discipline '{}'. Use: singles, ladies, pairs, or ice-dance",
                            discipline
                        ));
                    }
                };

                println!("ISU Validation ({:?})", disc);
                println!("-------------------");

                let result = validate_program(&program, disc);
                if result.errors.is_empty() && result.warnings.is_empty() {
                    println!("  All rules passed!");
                } else {
                    for error in &result.errors {
                        println!("  ERROR: {}", error);
                    }
                    for warning in &result.warnings {
                        println!("  WARNING: {}", warning);
                    }
                }
                println!();

                // Show segment rules
                for segment in &program.segments {
                    let rules = disc.segment_rules(segment.kind);
                    println!("Segment '{}' ({}) limits:", segment.name.node, segment.kind);
                    if let Some(max) = rules.max_jumps {
                        println!("  Max jumps: {}", max);
                    }
                    if let Some(max) = rules.max_spins {
                        println!("  Max spins: {}", max);
                    }
                    if let Some(count) = rules.step_sequences {
                        println!("  Step sequences: {}", count);
                    }
                    if let Some(count) = rules.required_lifts {
                        println!("  Lifts: {}", count);
                    }
                    if let Some(count) = rules.required_throws {
                        println!("  Throws: {}", count);
                    }
                    if let Some(count) = rules.required_twists {
                        println!("  Twists: {}", count);
                    }
                    if let Some(count) = rules.required_death_spirals {
                        println!("  Death spirals: {}", count);
                    }
                    println!();
                }
            }

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
