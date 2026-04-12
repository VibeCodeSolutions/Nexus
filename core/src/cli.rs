use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nexus", about = "NEXUS Personal ADHS-OS — Core Daemon")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the NEXUS server (default)
    Serve,
    /// Store an API key in the OS keychain
    SetKey {
        /// Provider name: claude or gemini
        provider: String,
        /// The API key value
        value: String,
    },
    /// Show QR code for Android pairing
    Pair,
}
