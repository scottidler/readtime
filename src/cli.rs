use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "readtime",
    about = "Estimate reading time for text documents",
    version = env!("GIT_DESCRIBE"),
    after_help = "Logs are written to: ~/.local/share/readtime/logs/readtime.log"
)]
pub struct Cli {
    /// Paths to files or directories (defaults to current directory)
    #[arg(help = "Paths to files or directories to analyze (defaults to current directory)")]
    pub paths: Vec<PathBuf>,

    /// Words per minute reading speed
    #[arg(short, long, default_value = "200", help = "Words per minute reading speed")]
    pub wpm: usize,

    /// File extensions to include (replaces defaults)
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "File extensions to include (comma-separated)"
    )]
    pub extensions: Option<Vec<String>>,

    /// Additional extensions to add to defaults
    #[arg(
        long,
        value_delimiter = ',',
        help = "Additional extensions to add to defaults (comma-separated)"
    )]
    pub add_extensions: Option<Vec<String>>,

    /// Path to config file
    #[arg(short, long, help = "Path to config file")]
    pub config: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,
}
