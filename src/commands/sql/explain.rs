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
            "**Discord Mapping**: Creates a new text channel within a database category with optional schema\n\n\
            **Process**:\n\
            • Requires active database selection (`USE <db>`)\n\
            • Creates text channel with name format: `table_<table_name>`\n\
            • Places channel inside the current database category\n\
            • Accepts SQL-like column definitions with constraints\n\
            • Stores complete schema information in channel topic\n\
            • Prevents duplicate table creation\n\n\
            **Syntax**: `/sql create table name:<table_name> [schema:<column_definitions>]`\n\n\
            **Examples**:\n\
            • Basic: `/sql create table customers`\n\
            • With schema: `/sql create table users id INT PRIMARY KEY, name VARCHAR(50), active BOOLEAN`\n\
            • Complex: `/sql create table products id INT, name VARCHAR(100), price DECIMAL, description VARCHAR(255)`\n\n\
            **Supported Data Types**:\n\
            • **INT**, **INTEGER** - Integer numbers\n\
            • **VARCHAR(size)**, **CHAR(size)** - Text with size limits (size validation enforced)\n\
            • **BOOLEAN**, **BOOL** - True/false values\n\
            • **FLOAT**, **DOUBLE**, **DECIMAL** - Decimal numbers\n\
            • **DATE**, **TIME**, **DATETIME** - Date and time values (stored as strings)\n\n\
            **Constraints**:\n\
            • **PRIMARY KEY** - Enforces uniqueness, prevents duplicate insertions\n\
            • **VARCHAR(n)/CHAR(n)** - String length validation (rejects strings longer than n)\n\
            • **NOT NULL** - Prevents null values (planned feature)\n\n\
            **Schema Storage**: Complete schema including constraints stored in Discord channel topic:\n\
            • Format: `Schema: id INT PRIMARY KEY, name VARCHAR(50), active BOOLEAN`\n\
            • Preserves PRIMARY KEY flags and size constraints\n\
            • Backward compatible with legacy formats\n\n\
            **Default Sizes**: VARCHAR defaults to 255, CHAR defaults to 1 if no size specified\n\n\
            **Result**: In `db_sales`, creates channel `table_customers` with validated schema"
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
            "� SELECT",
            "**Discord Mapping**: Queries data from table channels by reading stored messages\n\n\
            **Process**:\n\
            • Requires active database selection (`USE <db>`)\n\
            • Reads stored data from table channel messages\n\
            • Supports column selection, filtering, and DISTINCT\n\
            • Validates column names against table schema\n\
            • Returns formatted results in embed tables\n\n\
            **Syntax**: `/sql select columns:<cols> from:<table> [distinct:true] [where:<condition>] [params:<values>]`\n\n\
            **Column Selection**:\n\
            • All columns: `columns:*`\n\
            • Specific columns: `columns:id, name, email`\n\
            • Must match schema column names (if schema exists)\n\n\
            **Examples**:\n\
            • All data: `/sql select columns:* from:users`\n\
            • Specific columns: `/sql select columns:name, email from:customers`\n\
            • Single condition: `/sql select columns:* from:products where:price='100'`\n\
            • AND logic: `/sql select columns:* from:users where:name='John' AND age='25'`\n\
            • OR logic: `/sql select columns:* from:users where:name='John' OR name='Jane'`\n\
            • Parentheses grouping: `/sql select columns:* from:users where:(name='John' OR name='Jane') AND age='25'`\n\
            • Complex logic: `/sql select columns:* from:products where:category='Electronics' AND (price='100' OR price='200')`\n\
            • Nested grouping: `/sql select columns:* from:users where:(role='Admin' OR role='Manager') AND (department='IT' OR department='Sales')`\n\
            • Distinct values: `/sql select columns:category from:products distinct:true`\n\n\
            **Enhanced WHERE Conditions**:\n\
            • Single condition: `column_name='value'`\n\
            • AND logic: `col1='value1' AND col2='value2'` (both must be true)\n\
            • OR logic: `col1='value1' OR col2='value2'` (either can be true)\n\
            • **Parentheses grouping**: `(col1='value1' OR col2='value2') AND col3='value3'`\n\
            • **Nested conditions**: `(A AND B) OR (C AND D)` for complex logic\n\
            • **Operator Precedence**: Parentheses > AND > OR\n\
            • **Example Logic**: `A OR B AND C` evaluates as `A OR (B AND C)`, but `(A OR B) AND C` forces different grouping\n\n\
            **Features**:\n\
            • Schema validation for column names\n\
            • DISTINCT filtering to remove duplicates\n\
            • Dynamic table formatting (adapts column widths to content)\n\
            • Supports up to 20 rows in display (larger results truncated)\n\
            • Proper NULL, string, number, and boolean formatting\n\
            • Full AND/OR/parentheses logic support in WHERE clauses\n\n\
            **Result**: Formatted table showing selected data with query statistics"
        ),
        "insert" => (
            "➕ INSERT INTO",
            "**Discord Mapping**: Adds validated data to table channels as formatted messages\n\n\
            **Process**:\n\
            • Requires active database selection (`USE <db>`)\n\
            • Parses and validates SQL VALUES format\n\
            • Validates data against table schema (if defined)\n\
            • Enforces PRIMARY KEY uniqueness constraints\n\
            • Validates VARCHAR/CHAR length limits\n\
            • Checks data types and value formats\n\
            • Stores data as timestamped message in table channel\n\n\
            **Syntax**: `/sql insert into:<table_name> data:<values>`\n\n\
            **Data Format**: SQL VALUES format with proper type validation\n\n\
            **Examples**:\n\
            • Basic types: `/sql insert users 1, 'John Doe', 25, true`\n\
            • With constraints: `/sql insert products 'Widget', 'A useful tool'` (respects VARCHAR limits)\n\
            • Primary key table: `/sql insert customers 1, 'Alice'` (prevents duplicates)\n\n\
            **Supported Value Types**:\n\
            • Numbers: `42`, `3.14`, `-5` (validated as INT/FLOAT)\n\
            • Strings: `'John Doe'`, `'Hello World'` (single quotes, SQL standard)\n\
            • Booleans: `true`, `false`\n\
            • NULL: `NULL`\n\
            • Escaped quotes: `'It''s working!'`\n\n\
            **Schema Validation**:\n\
            • **Type checking**: INT, VARCHAR, CHAR, BOOLEAN, FLOAT, DOUBLE, DECIMAL, DATE, TIME, DATETIME\n\
            • **String length limits**: VARCHAR(50) rejects strings longer than 50 characters\n\
            • **Primary key constraints**: Prevents duplicate primary key values across all rows\n\
            • **Value count matching**: Must provide exactly the right number of values for schema columns\n\
            • **Detailed error messages**: Clear explanations with examples when validation fails\n\n\
            **Constraint Examples**:\n\
            • Length check: `name VARCHAR(10)` rejects `'ThisIsTooLongForTen'`\n\
            • Primary key: `id INT PRIMARY KEY` prevents inserting duplicate IDs\n\
            • Type validation: `age INT` rejects `'not_a_number'`\n\n\
            **Storage Format**: Data stored as structured message:\n\
            ```\n\
            TIMESTAMP: 2025-08-19 12:34:56 UTC\n\
            DATA:\n\
              id: 1\n\
              name: 'John Doe'\n\
              active: true\n\
            ```\n\n\
            **Backward Compatibility**: Handles tables created with legacy schema formats automatically"
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
            "**Available Commands** (Full Feature Status):\n\
            • ✅ `CREATE DATABASE` - Create database categories with permission management\n\
            • ✅ `DROP DATABASE` - Delete empty database categories safely\n\
            • ✅ `USE <database>` - Select current working database (per-user context)\n\
            • ✅ `CREATE TABLE` - Create tables with full schema support and constraints\n\
            • ✅ `DROP TABLE` - Delete tables and all their data permanently\n\
            • ✅ `SELECT` - **FULLY IMPLEMENTED** - Query data with column selection, filtering, and DISTINCT\n\
            • ✅ `INSERT` - **FULLY IMPLEMENTED** - Add validated data with comprehensive constraint checking\n\
            • 🚧 `UPDATE` - Modify existing data (planned feature)\n\
            • 🚧 `DELETE` - Remove data with conditions (planned feature)\n\n\
            **🚀 Advanced Features Implemented**:\n\
            • ✅ **Complete schema validation** - INT, VARCHAR(n), CHAR(n), BOOLEAN, FLOAT, DOUBLE, DECIMAL, DATE, TIME, DATETIME\n\
            • ✅ **PRIMARY KEY constraints** - Automatic uniqueness enforcement across all table rows\n\
            • ✅ **VARCHAR/CHAR length validation** - String size limits enforced on insertion\n\
            • ✅ **Column selection and filtering** - SELECT with *, specific columns, WHERE conditions\n\
            • ✅ **DISTINCT queries** - Remove duplicate rows from results\n\
            • ✅ **SQL-compliant formatting** - Single quotes for strings, proper NULL handling\n\
            • ✅ **Comprehensive error messages** - Detailed validation errors with helpful examples\n\
            • ✅ **Backward compatibility** - Automatic handling of legacy table formats\n\
            • ✅ **Formatted result display** - Professional table output with row numbers and statistics\n\n\
            **� Query Examples**:\n\
            • Get all data: `/sql select columns:* from:users`\n\
            • Specific columns: `/sql select columns:id, name, email from:customers`\n\
            • With filtering: `/sql select columns:name from:products where:price = '29.99'`\n\
            • Unique values: `/sql select columns:category from:products distinct:true`\n\n\
            **💾 Data Examples**:\n\
            • Insert with validation: `/sql insert users 1, 'John Doe', 'john@email.com'`\n\
            • Primary key protection: Duplicate IDs automatically rejected\n\
            • Length validation: VARCHAR(50) rejects strings longer than 50 characters\n\n\
            💡 **Quick Help**:\n\
            • `/sql explain create table` - Schema and constraint details\n\
            • `/sql explain insert` - Data validation and constraint enforcement\n\
            • `/sql explain select` - Querying and filtering capabilities"
        )
    };
    
    Ok(create_info_embed(title, description))
}
