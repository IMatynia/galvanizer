use clap::{Args, CommandFactory, Parser, Subcommand, error::ErrorKind};
use galvanizer_cli::{
    cli_errors::CLIError,
    commands::{backup, restore},
    configuration_loading::{ConfigLoadingErrors, load_app_config},
};
use std::{path::PathBuf, process::exit};

#[derive(Debug, Parser)]
#[command(name = "galvanizer")]
#[command(about = "A simple to use backup program", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration path, by default it is located in ~/.galvanizer.toml
    #[arg(long, env = "GALVANIZER_CONFIG")]
    config_path: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a new snapshot and run the backup
    Backup,
    /// Restore selected file or whole root or all roots
    Restore {
        /// Snapshot id to restore from
        snapshot_id: String,
        /// If defined, restores only this root
        root_id: Option<String>,
        /// If defined, restores only this file of the root
        file_id: Option<String>,
    },
    /// List information related to the data store and config
    List(ListArgs),
    /// Removes all entried from the data store that are not associated with any snapshot
    Prune(PruneArgs),
}

#[derive(Debug, Args)]
struct ListArgs {
    #[command(subcommand)]
    command: ListCommands,
}

#[derive(Debug, Subcommand)]
enum ListCommands {
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

#[derive(Debug, Args)]
struct PruneArgs {
    #[command(subcommand)]
    command: PruneCommands,
}

#[derive(Debug, Subcommand)]
enum PruneCommands {
    /// Remove data that is not assigned to any of the snapshots
    UnusedData,
    /// Keeps last N snapshots
    KeepSnapshots { n: u32 },
}

pub fn handle_config_errors(error: ConfigLoadingErrors) -> ! {
    let mut command = Cli::command();
    match error {
        ConfigLoadingErrors::CouldNotAccessHomeDirectory => command.error(
            ErrorKind::Io,
            "Could not access home directory! Specify a configuration location.",
        ),
        ConfigLoadingErrors::ConfigReadIOError(error) => command.error(
            ErrorKind::Io,
            format!("Could not read config file: {error}"),
        ),
        ConfigLoadingErrors::ConfigWriteIOError(error) => command.error(
            ErrorKind::Io,
            format!("Could not write config file: {error}"),
        ),
        ConfigLoadingErrors::ConfigDeseralizationError(error) => command.error(
            ErrorKind::InvalidValue,
            format!("Could not deserialize config file: {error}"),
        ),
        ConfigLoadingErrors::DefaultConfigError(config_error) => command.error(
            ErrorKind::InvalidValue,
            format!("An error occured while preparing the default configuration: {config_error:?}"),
        ),
        ConfigLoadingErrors::ConfigSerializationError(error) => command.error(
            ErrorKind::InvalidValue,
            format!("Could not serialize config file: {error}"),
        ),
    }
    .exit()
}

pub fn handle_cli_error(error: CLIError) -> ! {
    eprintln!("An error has occured: {error:?}");
    exit(1);
}

fn run(cli: Cli) {
    env_logger::init();

    dbg!(&cli);

    // configuration
    let config = match load_app_config(cli.config_path) {
        Ok(c) => c,
        Err(e) => handle_config_errors(e),
    };

    dbg!(&config);

    // cli commands parsing
    if let Err(e) = match cli.command {
        Commands::Backup => backup::run(config),
        Commands::Restore {
            snapshot_id,
            root_id,
            file_id,
        } => restore::run(config, snapshot_id, root_id, file_id),
        Commands::List(list_args) => match list_args.command {
            ListCommands::Snapshots => todo!(),
            ListCommands::Roots { snapshot_id } => todo!(),
            ListCommands::File {
                snapshot_id,
                root_id,
            } => todo!(),
        },
        Commands::Prune(prune_args) => match prune_args.command {
            PruneCommands::UnusedData => todo!(),
            PruneCommands::KeepSnapshots { n } => todo!(),
        },
    } {
        handle_cli_error(e);
    }
}

fn main() {
    let cli = Cli::parse();
    run(cli);
}
