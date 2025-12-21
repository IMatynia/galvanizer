use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};
use galvanizer_cli::{
    cli_errors::CLIError,
    commands::{backup::run::backup_command, list, restore},
    configuration_loading::{ConfigLoadingErrors, load_app_config},
    first_time_config_prompt::first_time_config_customization_prompt,
    schemas::{
        list::ListArgs,
        prune::{PruneArgs, PruneCommands},
        restore::RestoreArgs,
    },
};
use std::{env::home_dir, path::PathBuf, process::exit};

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
    Restore(RestoreArgs),
    /// List information related to the data store and config
    List(ListArgs),
    /// Removes all entried from the data store that are not associated with any snapshot
    Prune(PruneArgs),
    /// Reopen initial config setup dialog
    NewConfig,
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
fn default_config_path() -> Option<PathBuf> {
    home_dir().map(|x| x.join(".galvanizer.toml"))
}
fn run(cli: Cli) {
    env_logger::init();

    dbg!(&cli);

    // configuration
    let config_path = cli
        .config_path
        .or(default_config_path())
        .expect("Failed to query for a default config location in your home directory!");

    let config = match load_app_config(config_path.clone()) {
        Ok(c) => c,
        Err(e) => handle_config_errors(e),
    };

    dbg!(&config);

    // cli commands parsing
    if let Err(e) = match cli.command {
        Commands::Backup => backup_command(config),
        Commands::Restore(args) => restore::run(config, args),
        Commands::List(list_args) => list::run(config, list_args.command),
        Commands::Prune(prune_args) => match prune_args.command {
            PruneCommands::UnusedData => todo!(),
            PruneCommands::KeepSnapshots { n } => todo!(),
        },
        Commands::NewConfig => {
            if let Err(e) = first_time_config_customization_prompt(&config_path) {
                handle_config_errors(e);
            }
            Ok(())
        }
    } {
        handle_cli_error(e);
    }
}

fn main() {
    let cli = Cli::parse();
    run(cli);
}
