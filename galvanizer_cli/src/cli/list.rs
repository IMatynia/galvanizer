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
    Roots { snapshot_id: String },
    /// List all files from a root in a snapshot
    File {
        snapshot_id: String,
        root_id: String,
    },
}
