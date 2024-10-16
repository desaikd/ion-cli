pub mod load;
pub mod validate;

use crate::commands::CommandRunner;
use anyhow::Result;
use clap::{ArgMatches, Command};

// To add a schema subcommand, add your new command to the `schema_subcommands`
// and `runner_for_schema_subcommands` functions.

// Creates a Vec of CLI configurations for all of the available built-in subcommands for schema
pub fn schema_subcommands() -> Vec<Command> {
    vec![load::app(), validate::app()]
}

pub fn runner_for_schema_subcommand(command_name: &str) -> Option<CommandRunner> {
    let runner = match command_name {
        "load" => load::run,
        "validate" => validate::run,
        _ => return None,
    };
    Some(runner)
}

// The functions below are used by the `beta` subcommand when `schema` is invoked.
pub fn run(_command_name: &str, matches: &ArgMatches) -> Result<()> {
    // We want to evaluate the name of the subcommand that was invoked
    let (command_name, command_args) = matches.subcommand().unwrap();
    if let Some(runner) = runner_for_schema_subcommand(command_name) {
        // If a runner is registered for the given command name, command_args is guaranteed to
        // be defined; we can safely unwrap it.
        runner(command_name, command_args)?;
    } else {
        let message = format!(
            "The requested schema command ('{}') is not supported and clap did not generate an error message.",
            command_name
        );
        unreachable!("{}", message);
    }
    Ok(())
}

pub fn app() -> Command {
    Command::new("schema")
        .about(
            "The 'schema' command is a namespace for commands that are related to schema sandbox",
        )
        .subcommands(schema_subcommands())
}
