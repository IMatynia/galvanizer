use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct PruneArgs {
    #[command(subcommand)]
    pub command: PruneCommands,
}

#[derive(Debug, Subcommand)]
pub enum PruneCommands {
    /// Remove data that is not assigned to any of the snapshots
    UnusedData,
    /// Keeps last N snapshots
    KeepSnapshots { n: usize },
}
