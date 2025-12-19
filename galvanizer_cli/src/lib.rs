pub mod cli;
pub mod cli_command_handlers;
pub mod cli_errors;
pub mod commands;
pub mod configuration_loading;
pub mod first_time_config_prompt;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION"); 

#[cfg(test)]
mod tests;
