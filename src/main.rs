use clap::Parser;
use colored::*;
use eyre::{Context, Result};
use log::info;
use std::fs;
use std::path::PathBuf;

mod cli;
mod config;
mod counter;
mod estimator;
mod tree;
mod walker;

use cli::Cli;
use config::Config;

fn setup_logging() -> Result<()> {
    // Create log directory
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("readtime")
        .join("logs");

    fs::create_dir_all(&log_dir).context("Failed to create log directory")?;

    let log_file = log_dir.join("readtime.log");

    // Setup env_logger with file output
    let target = Box::new(
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .context("Failed to open log file")?,
    );

    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(target))
        .init();

    info!("Logging initialized, writing to: {}", log_file.display());
    Ok(())
}

fn run_application(cli: &Cli, config: &Config) -> Result<()> {
    info!("Starting readtime analysis");

    // Determine which extensions to use
    let extensions = if let Some(ref exts) = cli.extensions {
        // User specified extensions, use those
        exts.clone()
    } else if let Some(ref add_exts) = cli.add_extensions {
        // Add to default extensions
        let mut exts = config.extensions.clone();
        exts.extend(add_exts.clone());
        exts
    } else {
        // Use default from config
        config.extensions.clone()
    };

    // Determine paths to process (default to current directory)
    let paths: Vec<PathBuf> = if cli.paths.is_empty() { vec![PathBuf::from(".")] } else { cli.paths.clone() };

    if cli.verbose {
        println!("{} Processing: {:?}", "→".cyan(), paths);
        println!("{} Extensions: {:?}", "→".cyan(), extensions);
        println!("{} WPM: {}", "→".cyan(), cli.wpm);
    }

    // Process all paths
    let mut all_files = Vec::new();
    for path in &paths {
        match walker::process_path(path, &extensions, cli.wpm) {
            Ok(files) => all_files.extend(files),
            Err(e) => {
                if cli.verbose {
                    eprintln!("{} Skipping {}: {}", "⚠".yellow(), path.display(), e);
                }
            }
        }
    }

    if all_files.is_empty() {
        println!("{} No matching files found", "✗".red());
        return Ok(());
    }

    info!("Processed {} files", all_files.len());

    // If single path, build tree for that path
    // If multiple paths, show flat list or multiple trees
    if paths.len() == 1 {
        let tree = tree::build_tree(&all_files, &paths[0]);
        let output = tree::render_tree(&tree, None);
        print!("{}", output);
    } else {
        // Multiple paths: show separate tree for each
        for path in &paths {
            let path_files: Vec<_> = all_files.iter().filter(|f| f.path.starts_with(path)).cloned().collect();

            if !path_files.is_empty() {
                let tree = tree::build_tree(&path_files, path);
                let output = tree::render_tree(&tree, None);
                print!("{}", output);
            }
        }
    }

    info!("Analysis complete");
    Ok(())
}

fn main() -> Result<()> {
    // Setup logging first
    setup_logging().context("Failed to setup logging")?;

    // Parse CLI arguments
    let cli = Cli::parse();

    // Load configuration
    let config = Config::load(cli.config.as_ref()).context("Failed to load configuration")?;

    info!("Starting with config from: {:?}", cli.config);

    // Run the main application logic
    run_application(&cli, &config).context("Application failed")?;

    Ok(())
}
