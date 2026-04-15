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
    /// Interaktiver Wizard: Provider + Auth-Verfahren wählen
    Onboard,
    /// OAuth-Login (nur Claude); öffnet Browser
    Login {
        /// Provider — aktuell nur "claude"
        #[arg(default_value = "claude")]
        provider: String,
    },
    /// Store an API key in the OS keychain
    SetKey {
        /// Provider name: claude or gemini
        provider: String,
        /// The API key value
        value: String,
    },
    /// OAuth-Token oder API-Key löschen
    Logout {
        /// Provider name
        provider: String,
    },
    /// Show QR code for Android pairing
    Pair,
    /// Diagnose: zeigt welche Provider/Auth-Methoden konfiguriert sind
    Status,
    /// Test-Call gegen den aktuell aktiven LLM-Provider
    TestLlm {
        /// Optional: Provider überschreiben (sonst default aus config)
        #[arg(long)]
        provider: Option<String>,
        /// Optional: Test-Text
        #[arg(long, default_value = "Ich sollte morgen die Steuererklärung machen.")]
        text: String,
    },
}
