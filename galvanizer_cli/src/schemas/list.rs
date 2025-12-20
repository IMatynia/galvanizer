use clap::{Args, Subcommand, command};

#[derive(Debug, Args)]
pub struct ListArgs {
    #[command(subcommand)]
    pub command: ListCommands,
}

#[derive(Debug, Subcommand)]
pub enum ListCommands {
    /// List all snapshots
    Snapshots,
    /// List all roots of a given snapshot
    Roots {
        /// If no snapshot is defined, the newest one will be used
        snapshot_id: Option<String>,
    },
    /// List all files from a root in a snapshot
    Files {
        /// If no snapshot is defined, the newest one will be used
        root_id: String,
        snapshot_id: Option<String>,
    },
}
