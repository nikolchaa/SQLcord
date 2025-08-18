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
