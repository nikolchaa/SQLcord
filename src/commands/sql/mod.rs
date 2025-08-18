// SQL command group
pub mod create;
pub mod drop;

pub mod use_;
pub mod select;
pub mod insert;
pub mod update;
pub mod delete;
pub mod explain;

use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandOptionType;
use std::error::Error;
use crate::logging::{log_info, log_error};

/// Register all SQL commands dynamically
pub fn register_all_sql_commands() -> Result<(), Box<dyn Error>> {
    log_info("Starting SQL command registration system...");
    
    // Register subcommand groups
    if let Err(e) = create::register_create_subcommands() {
        log_error(&format!("Failed to register CREATE subcommands: {}", e));
        return Err(e);
    }
    
    if let Err(e) = drop::register_drop_subcommands() {
        log_error(&format!("Failed to register DROP subcommands: {}", e));
        return Err(e);
    }
    
    // Register individual commands
    log_info("Registering individual SQL commands...");
    
    if let Err(e) = use_::register() {
        log_error(&format!("Failed to register USE command: {}", e));
        return Err(e);
    }
    
    if let Err(e) = select::register() {
        log_error(&format!("Failed to register SELECT command: {}", e));
        return Err(e);
    }
    
    if let Err(e) = insert::register() {
        log_error(&format!("Failed to register INSERT command: {}", e));
        return Err(e);
    }
    
    if let Err(e) = update::register() {
        log_error(&format!("Failed to register UPDATE command: {}", e));
        return Err(e);
    }
    
    if let Err(e) = delete::register() {
        log_error(&format!("Failed to register DELETE command: {}", e));
        return Err(e);
    }
    
    if let Err(e) = explain::register() {
        log_error(&format!("Failed to register EXPLAIN command: {}", e));
        return Err(e);
    }
    
    log_info("All SQL commands registered successfully!");
    Ok(())
}

/// Build and return the `/sql` CreateCommand builder.
pub fn register_sql_group() -> CreateCommand {
    CreateCommand::new("sql").description("Run SQL-like operations mapped to Discord")
        // create group: /sql create db <name>
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommandGroup, "create", "Create resources")
                .set_sub_options(vec![
                    CreateCommandOption::new(CommandOptionType::SubCommand, "db", "Create a database (category)")
                        .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "Database name").required(true)),
                    CreateCommandOption::new(CommandOptionType::SubCommand, "table", "Create a table (channel)")
                        .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "Table name").required(true))
                ])
        )
        // drop group: /sql drop db <name>
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommandGroup, "drop", "Drop resources")
                .set_sub_options(vec![
                    CreateCommandOption::new(CommandOptionType::SubCommand, "db", "Drop a database (category)")
                        .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "Database name").required(true)),
                    CreateCommandOption::new(CommandOptionType::SubCommand, "table", "Drop a table (channel)")
                        .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "Table name").required(true))
                ])
        )
        // use subcommand: /sql use <name>
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "use", "Select database to use")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "Database name").required(true))
        )
        // lightweight placeholders for data ops
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "select", "Read rows from a table")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "table", "Table name").required(true))
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "insert", "Insert a row into a table")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "table", "Table name").required(true))
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "data", "Row data (json or kv)").required(true))
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "update", "Update rows in a table")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "table", "Table name").required(true))
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "delete", "Delete rows from a table")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "table", "Table name").required(true))
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "explain", "Explain an operation")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "op", "Operation to explain").required(true))
        )
}
