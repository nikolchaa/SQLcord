// /sql explain <operation>

use std::error::Error;
use serenity::builder::CreateEmbed;
use crate::logging::log_info;
use crate::utils::create_info_embed;

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering EXPLAIN command");
    Ok(())
}

/// Explain how SQL operations are mapped to Discord operations
/// Returns an info embed with detailed explanations
pub async fn run(operation: &str) -> Result<CreateEmbed, CreateEmbed> {
    log_info(&format!("EXPLAIN command executed for operation: {}", operation));
    
    let operation_lower = operation.to_lowercase();
    let (title, description) = match operation_lower.as_str() {
        "create database" | "create_database" => (
            "📁 CREATE DATABASE",
            "**Discord Mapping**: Creates a new Discord category channel\n\n\
            **Process**:\n\
            • Creates category with name format: `db_<database_name>`\n\
            • Sets up permissions for the bot to manage channels\n\
            • Prevents duplicate database creation\n\n\
            **Example**: `CREATE DATABASE sales` → Category: `db_sales`"
        ),
        "drop database" | "drop_database" => (
            "🗑️ DROP DATABASE", 
            "**Discord Mapping**: Deletes a Discord category channel\n\n\
            **Process**:\n\
            • Locates category with name format: `db_<database_name>`\n\
            • Checks if database contains tables (prevents deletion)\n\
            • Removes empty database categories only\n\n\
            **Safety**: Non-empty databases cannot be deleted"
        ),
        "create table" | "create_table" => (
            "📋 CREATE TABLE",
            "**Discord Mapping**: Creates a new text channel within a database category\n\n\
            **Process**:\n\
            • Requires active database selection (`USE <db>`)\n\
            • Creates text channel with name format: `table_<table_name>`\n\
            • Places channel inside the current database category\n\
            • Prevents duplicate table creation\n\n\
            **Example**: In `db_sales`, `CREATE TABLE customers` → Channel: `table_customers`"
        ),
        "drop table" | "drop_table" => (
            "🗑️ DROP TABLE",
            "**Discord Mapping**: Deletes a text channel from the current database\n\n\
            **Process**:\n\
            • Requires active database selection (`USE <db>`)\n\
            • Locates text channel with name format: `table_<table_name>`\n\
            • Removes the channel and any stored data\n\n\
            **Warning**: This permanently deletes the table and all data"
        ),
        "use" | "use database" => (
            "🎯 USE DATABASE",
            "**Discord Mapping**: Sets your current working database context\n\n\
            **Process**:\n\
            • Stores your selected database in session memory\n\
            • Validates that the database category exists\n\
            • Required before creating/dropping tables\n\n\
            **Session**: Each user has their own database context per server"
        ),
        "select" => (
            "🔍 SELECT (Future)",
            "**Discord Mapping**: Will query data from table channels\n\n\
            **Planned Process**:\n\
            • Read data stored in channel topic/messages\n\
            • Support filtering and sorting operations\n\
            • Return results in formatted embeds\n\n\
            **Status**: Not yet implemented"
        ),
        "insert" => (
            "➕ INSERT (Future)",
            "**Discord Mapping**: Will add data to table channels\n\n\
            **Planned Process**:\n\
            • Store data in channel messages or topic\n\
            • Support multiple data formats\n\
            • Validate data types and constraints\n\n\
            **Status**: Not yet implemented"
        ),
        "update" => (
            "✏️ UPDATE (Future)",
            "**Discord Mapping**: Will modify existing data in table channels\n\n\
            **Planned Process**:\n\
            • Locate and modify specific records\n\
            • Support conditional updates\n\
            • Maintain data history if needed\n\n\
            **Status**: Not yet implemented"
        ),
        "delete" => (
            "❌ DELETE (Future)",
            "**Discord Mapping**: Will remove data from table channels\n\n\
            **Planned Process**:\n\
            • Remove specific records from tables\n\
            • Support conditional deletion\n\
            • Maintain referential integrity\n\n\
            **Status**: Not yet implemented"
        ),
        _ => (
            "❓ Unknown Operation",
            "**Available Commands**:\n\
            • `CREATE DATABASE` - Create a new database\n\
            • `DROP DATABASE` - Delete an empty database\n\
            • `USE <database>` - Select current database\n\
            • `CREATE TABLE` - Create a new table\n\
            • `DROP TABLE` - Delete a table\n\
            • `SELECT` - Query data (coming soon)\n\
            • `INSERT` - Add data (coming soon)\n\
            • `UPDATE` - Modify data (coming soon)\n\
            • `DELETE` - Remove data (coming soon)\n\n\
            **Tip**: Try `/sql explain create table` for specific help"
        )
    };
    
    Ok(create_info_embed(title, description))
}
