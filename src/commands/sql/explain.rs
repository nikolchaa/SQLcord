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
            • Optionally accepts SQL-like column definitions\n\
            • Stores schema information in channel topic\n\
            • Prevents duplicate table creation\n\n\
            **Examples**:\n\
            • Basic: `CREATE TABLE customers`\n\
            • With schema: `CREATE TABLE Persons (PersonID int, LastName varchar(255), FirstName varchar(255), Address varchar(255), City varchar(255))`\n\n\
            **Supported Data Types**: INT, VARCHAR, CHAR, BOOLEAN, FLOAT, DOUBLE, DECIMAL, DATE, TIME, DATETIME\n\
            Only these types are accepted. Any other type will result in an error.\n\n\
            **Default Sizes**: VARCHAR defaults to 255, CHAR defaults to 1 if no size specified.\n\n\
            **Result**: In `db_sales`, creates channel `table_customers` with schema stored in topic"
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
            "➕ INSERT INTO",
            "**Discord Mapping**: Adds data to table channels as messages\n\n\
            **Process**:\n\
            • Requires active database selection (`USE <db>`)\n\
            • Validates SQL VALUES format and data types\n\
            • Checks data against table schema (if defined)\n\
            • Validates required fields and constraints\n\
            • Stores data as timestamped message in table channel\n\n\
            **Syntax**: `/sql insert into table:<table_name> data:<values>`\n\n\
            **Data Format**: SQL VALUES format\n\
            **Examples**:\n\
            • Basic: `/sql insert into:users data:1, 'John', 25`\n\
            • With types: `/sql insert into:products data:101, 'Widget', 29.99, true`\n\
            • With NULL: `/sql insert into:customers data:1, 'Alice', NULL, false`\n\n\
            **Supported Value Types**:\n\
            • Numbers: `42`, `3.14`, `-5`\n\
            • Strings: `'John Doe'`, `'Hello World'`\n\
            • Booleans: `true`, `false`\n\
            • NULL: `NULL`\n\
            • Escaped quotes: `'It''s working!'`\n\n\
            **Validation**:\n\
            • Type checking (INT, VARCHAR, BOOLEAN, etc.)\n\
            • String length limits (VARCHAR size)\n\
            • Required field validation (NOT NULL columns)\n\
            • Value count matching schema columns\n\n\
            **Result**: Data stored as message in `table_<name>` channel with timestamp"
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
            "✖️ DELETE (Future)",
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
            💡 **Tip**: Try `/sql explain create table` for specific help"
        )
    };
    
    Ok(create_info_embed(title, description))
}
