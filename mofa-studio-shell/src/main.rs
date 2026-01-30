//! MoFA Studio - Main entry point
//!
//! Parses command-line arguments and starts the application.
//!
//! # Usage
//!
//! ```bash
//! mofa-studio --help          # Show help
//! mofa-studio --dark-mode     # Start in dark mode
//! mofa-studio --log-level debug  # Enable debug logging
//! ```

mod app;
mod cli;

pub use cli::Args;

use clap::Parser;
use std::path::Path;

fn main() {
    // Parse command-line arguments
    let args = Args::parse();

    // Configure logging based on CLI args
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(args.log_filter()),
    )
    .init();

    log::info!("Starting MoFA Studio");
    log::debug!("CLI args: {:?}", args);

    if let Ok(dir) = std::env::var("MOFA_STUDIO_DIR") {
        let path = Path::new(&dir);
        if path.exists() {
            if let Err(err) = std::env::set_current_dir(path) {
                log::warn!("Failed to set current dir to {}: {}", dir, err);
            } else {
                log::info!("Working directory set to {}", dir);
            }
        }
    }

    if args.dark_mode {
        log::info!("Dark mode enabled via CLI");
    }

    if let Some(ref dataflow) = args.dataflow {
        log::info!("Using dataflow: {}", dataflow);
    }

    // Store args for app to access
    app::set_cli_args(args);

    // Start the application
    app::app_main();
}
