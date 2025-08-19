# 🥏 SQLcord - SQL prompts inside Discord (Rust)

A Discord bot for executing SQL commands in a fun and educational way. This project maps:

- Databases -> Discord categories (named `db_<name>`)
- Tables -> Channels inside the category (named `table_<name>`)
- Rows -> Messages inside the table channel

## Features ✨

### Implementation Status

- ✅ **Database creation** - Create and manage database categories
- ✅ **Table creation** - Create tables with optional schema definitions
- ✅ **Data insertion** - Insert validated data with schema support
- ✅ **Schema validation** - Type checking and constraint validation
- ✅ **Primary key constraints** - Uniqueness enforcement with duplicate prevention
- ✅ **VARCHAR length limits** - String size validation and enforcement
- ✅ **Backward compatibility** - Support for legacy schema formats
- ✅ **SQL conventions** - Proper single quote formatting for string values
- ✅ **Data querying** - SELECT operations with full WHERE logic (AND/OR/parentheses)
- 🚧 **Data modification** - UPDATE and DELETE operations (stubs exist)
- 🚧 **Advanced features** - Joins, indexing, transactions (future work)

### Supported Data Types

- `INT` - Integer values
- `VARCHAR(size)` - Variable-length strings with size limits
- `CHAR(size)` - Fixed-length strings with size limits
- `BOOLEAN` - True/false values
- `FLOAT`, `DOUBLE`, `DECIMAL` - Floating-point numbers
- `DATE`, `TIME`, `DATETIME` - Date and time values (must be valid ISO 8601 format)

### Constraints

- **PRIMARY KEY** - Ensures uniqueness across all rows in a table
- **VARCHAR/CHAR length limits** - Validates string lengths against defined sizes
- **Type validation** - Ensures data matches column types
- **NOT NULL** (planned) - Prevents null values in specified columns

## Quick start 🚀

1. Copy the template and set your bot token:

   - Copy `.env.example` to `.env` and set `DISCORD_TOKEN=your-token` (do not commit `.env`).

2. Invite the bot to your server:

   - In the Discord Developer Portal use OAuth2 → URL Generator:
     - Scopes: `bot` and `applications.commands`
     - Permissions: `Administrator` for convenience during dev

3. Run locally:

```bash
cargo run
```

## Testing 🧪

The project includes several tests to verify functionality.
To run the tests, simply execute:

```bash
cargo test
```

This command will automatically discover and run all tests in the project. Make sure your development environment is set up with Rust and all dependencies installed.

## Commands implemented 🛠️

- `/sql create db <name>` - creates a category named `db_<name>`.
- `/sql create table <name> [schema]` - creates a text channel named `table_<name>` under the current database category. Optionally accepts schema definitions.
- `/sql use <name>` - selects an existing `db_<name>` for your user (kept in-memory per guild+user).
- `/sql insert into <table> <data>` - inserts data into a table (Discord channel) with validation against the table schema.
- `/sql select columns:<columns> from:<table> [distinct:<true/false>] [where:<conditions>]` - queries data from a table with support for column selection, DISTINCT filtering, and advanced WHERE conditions with AND/OR/parentheses logic.

### Table Schema Support

SQLcord supports defining table schemas when creating tables:

```bash
/sql create table users id INT, name VARCHAR(255), active BOOLEAN
```

**Supported Data Types:**

- `INT`, `INTEGER` - Integer numbers (no size specification allowed)
- `VARCHAR(size)`, `CHAR(size)` - Text with required size limit (1-65535 characters)
- `BOOLEAN`, `BOOL` - True/false values (no size specification allowed)
- `FLOAT(precision)`, `DOUBLE(precision)`, `DECIMAL(precision)` - Decimal numbers with optional precision (1-65)
- `DATE`, `TIME`, `DATETIME` - Date and time values (must be valid ISO 8601 format, no size specification allowed)

**Schema Validation Rules:**

- **Required sizes**: `VARCHAR` and `CHAR` must specify size: `VARCHAR(255)`, `CHAR(10)`
- **No sizes allowed**: `INT`, `BOOLEAN`, `DATE`, `TIME`, `DATETIME` cannot have size specifications
- **Optional precision**: `FLOAT`, `DOUBLE`, `DECIMAL` can optionally specify precision: `DECIMAL(10)`
- **Size limits**: VARCHAR/CHAR sizes must be 1-65535, decimal precision must be 1-65
- **Clear error messages**: Detailed validation feedback with examples and suggestions

**Valid Schema Examples:**

```bash
# User management table
/sql create table users id INT, name VARCHAR(100), email VARCHAR(255), active BOOLEAN, created_at DATETIME

# Product catalog with pricing
/sql create table products id INT, name VARCHAR(50), price DECIMAL(10), description VARCHAR(1000), in_stock BOOLEAN

# System logs
/sql create table logs timestamp DATETIME, level VARCHAR(10), message VARCHAR(500), user_id INT

# Employee records
/sql create table employees id INT PRIMARY KEY, first_name VARCHAR(50), last_name VARCHAR(50), salary FLOAT, hire_date DATE
```

**Common Validation Errors:**

```bash
# ❌ Missing size specification
/sql create table users name VARCHAR
# Error: "VARCHAR requires a size specification"

# ❌ Invalid size on INT
/sql create table users id INT(11)
# Error: "INT does not support size specification"

# ❌ Invalid size on BOOLEAN
/sql create table users active BOOLEAN(1)
# Error: "BOOLEAN does not support size specification"

# ❌ Invalid size on DATE/TIME
/sql create table events created_at DATETIME(20)
# Error: "DATETIME does not support size specification"

# ❌ Zero or invalid size
/sql create table users name VARCHAR(0)
# Error: "VARCHAR size must be greater than 0"
```

**Date/Time Validation Errors:**

```bash
# ❌ Invalid DATE format (must be YYYY-MM-DD)
/sql insert into logs 'ASDASDASD', 'INFO', 'Test message'
# Error: "Value 'ASDASDASD' is not a valid ISO date. Use format: YYYY-MM-DD (e.g., '2025-01-15')"

# ❌ Invalid TIME format (must be HH:MM:SS with zero-padding)
/sql insert into events '2:30:00', 'Meeting'
# Error: "Value '2:30:00' is not a valid ISO time. Use format: HH:MM:SS[.fraction][Z|±HH:MM] (e.g., '14:30:00', '14:30:00.500Z')"

# ❌ Invalid DATETIME format (must be ISO 8601)
/sql insert into activities 'None', 'Test activity'
# Error: "Value 'None' is not a valid ISO datetime. Use format: YYYY-MM-DDTHH:MM:SS[.fraction][Z|±HH:MM] (e.g., '2025-01-15T14:30:00Z')"

# ❌ Invalid date values (like February 30th)
/sql insert into events '2025-02-30', 'Invalid date'
# Error: "Value '2025-02-30' is not a valid ISO date. Use format: YYYY-MM-DD (e.g., '2025-01-15')"
```

**Schema Features:**

- **Type validation** - INSERT commands validate data against the defined schema
  - **VARCHAR/CHAR length limits** - Enforces size constraints for string types
  - **ISO 8601 date/time validation** - DATE, TIME, and DATETIME columns require valid ISO format
  - **Numeric validation** - INT, FLOAT, DECIMAL values validated for correct format
  - **Boolean validation** - BOOLEAN columns accept only true/false values
- **Primary key constraints** - Prevents duplicate primary key values across rows
- **Flexible insertion** - Tables without schemas accept any data format
- **Backward compatibility** - Automatically handles tables created with older schema formats
- **Storage format** - Schemas are stored in Discord channel topics for persistence
- **SQL conventions** - String values displayed with single quotes following SQL standards

**Example INSERT with schema validation:**

```bash
/sql insert into users 1, 'John Doe', true
```

**Results in structured data storage:**

```
TIMESTAMP: 2025-08-19T00:14:00Z
DATA:
  id: 1
  name: 'John Doe'
  active: true
```

**Literal parsing rules and ISO examples**

- DATE / TIME / DATETIME examples (preferred canonical ISO forms):

  - DATE: `'2025-08-19'` → stored as string (use `YYYY-MM-DD`).
  - TIME: `'00:14:00'`, `'00:14:00.123'`, or with offset `'00:14:00Z'` (use `HH:MM:SS[.fraction][Z|±HH:MM]`).
  - DATETIME / TIMESTAMP: `'2025-08-19T00:14:00Z'` (preferred) or `'2025-08-19T00:14:00+00:00'` (use full ISO 8601 with `T` and `Z`/offset).

  **⚠️ Important**: When inserting into DATE/TIME/DATETIME columns, the values are strictly validated for ISO 8601 format. Invalid formats like `'ASDASDASD'`, `'None'`, or `'2:30:00'` (missing zero-padding) will be rejected with detailed error messages.

- Literal parsing rules (input → parsed `SqlValue`):

  - Strings: must be single-quoted. Use doubled single-quote to escape inside strings:
    - Input: `'it''s a test'` → `SqlValue::String("it's a test")`.
    - Backslashes are preserved literally in the stored string:
      - Input: `'C:\\path\\to\\file'` → `SqlValue::String("C:\\path\\to\\file")`.
  - NULL: the unquoted token `NULL` (case-insensitive) → `SqlValue::Null`.
  - Boolean: unquoted `true` / `false` (case-insensitive) → `SqlValue::Boolean(true/false)`.
  - Numbers: unquoted numeric tokens are parsed with precedence:
    1. Try integer parse first → `SqlValue::Integer(i64)` (e.g. `42` → `Integer(42)`).
    2. If integer parse fails but token is a decimal → `SqlValue::Float(f64)` (e.g. `3.14` → `Float(3.14)`).
  - Unquoted non-number/non-boolean tokens are rejected with an error; strings must be single-quoted.

- Quick input → parsed examples:
  - `'2025-08-19'` → `SqlValue::String("2025-08-19")` (DATE as string)
  - `'00:14:00'` → `SqlValue::String("00:14:00")` (TIME as string)
  - `'2025-08-19T00:14:00Z'` → `SqlValue::String("2025-08-19T00:14:00Z")` (DATETIME as string)
  - `'it''s'` → `SqlValue::String("it's")`
  - `'C:\\path\\file'` → `SqlValue::String("C:\\path\\file")`
  - `NULL` → `SqlValue::Null`
  - `true` / `TRUE` → `SqlValue::Boolean(true)`
  - `123` → `SqlValue::Integer(123)`
  - `123.0` → `SqlValue::Float(123.0)`
  - `foo` → parse error: "Unquoted token 'foo' is not a number, boolean, or NULL; string literals must be single-quoted (e.g. 'foo')"

### Primary Key Constraints

Tables can define primary key columns that enforce uniqueness:

```bash
/sql create table users id INT PRIMARY KEY, name VARCHAR(255), email VARCHAR(255)
```

**Primary Key Features:**

- **Uniqueness enforcement** - Prevents duplicate primary key values
- **Automatic validation** - Checks existing rows before allowing new inserts
- **Clear error messages** - Shows which column and value caused the violation

**Example of primary key violation:**

```bash
# First insert succeeds
/sql insert users 1, 'Alice', 'alice@example.com'

# Second insert with same ID fails
/sql insert users 1, 'Bob', 'bob@example.com'
# Error: "Duplicate primary key detected! Primary key column(s): id, Value(s): 1"
```

### Data Validation & Constraints

SQLcord provides comprehensive data validation:

**VARCHAR Length Validation:**

```bash
# Create table with VARCHAR size limit
/sql create table products name VARCHAR(10), description VARCHAR(100)

# This succeeds
/sql insert products 'Widget', 'A useful tool'

# This fails - name too long
/sql insert products 'SuperLongProductName', 'Description'
# Error: "String too long for column name (position 1). Length: 19 characters, Maximum: 10 characters"
```

**Type Validation:**

```bash
# Create table with typed columns
/sql create table orders id INT, amount FLOAT, completed BOOLEAN

# This succeeds
/sql insert orders 100, 29.99, true

# This fails - wrong type
/sql insert orders 'not_a_number', 29.99, true
# Error: "Type mismatch for column id (position 1). Expected: integer, Got: string"
```

**Primary Key Enforcement:**

```bash
# Create table with primary key
/sql create table customers id INT PRIMARY KEY, name VARCHAR(50)

# First insert succeeds
/sql insert customers 1, 'John'

# Duplicate primary key fails
/sql insert customers 1, 'Jane'
# Error: "Duplicate primary key detected! Primary key column(s): id, Value(s): 1"
```

### Dynamic Command Registration System

SQLcord features a dynamic command registration system that:

- **Automatically registers all commands** during bot startup
- **Provides detailed logging** with timestamps and consistent color coding
- **Shows registration status** for each command module (CREATE, DROP, individual SQL commands)
- **Handles errors gracefully** with clear failure messages

### Centralized Logging System

The project features a centralized logging system (`src/logging.rs`) that:

- **Consistent formatting** across all modules with proper timestamps
- **Color-coded output** using ANSI escape sequences for better readability
- **Unified log_info and log_error functions** used throughout the codebase instead of println!

The registration system prints formatted logs like:

```bash
2025-08-19T00:12:48.361173Z  INFO Starting dynamic registration of CREATE subcommands...
2025-08-19T00:12:48.361261Z  INFO Registering DB command...
2025-08-19T00:12:48.361359Z  INFO Registering CREATE DB command
2025-08-19T00:12:48.361442Z ERROR DB command registration failed.
```

## Examples 🎭

### Basic Database Operations

- **Create a database called `test`:**
  - Use the slash command: `/sql create db name:test`
  - Bot replies: "Database `db_test` created"
- **Use the database you just created:**
  - `/sql use name:test`
  - Bot replies: "Using database `db_test`"

### Table Operations with Schema

- **Create a table with comprehensive schema:**
  - `/sql create table name:users schema:id INT, name VARCHAR(255), email VARCHAR(100), active BOOLEAN, created_at DATETIME`
  - Bot creates a channel `table_users` with the defined schema and validation rules
- **Create a table with decimal precision:**
  - `/sql create table name:products schema:id INT, name VARCHAR(50), price DECIMAL(10), description VARCHAR(1000)`
  - Bot validates that all required size specifications are provided
- **Create a simple table without schema:**

  - `/sql create table name:logs`
  - Bot creates a flexible table that accepts any data format

- **Common schema validation examples:**

  ```bash
  # ✅ Valid schemas
  /sql create table events id INT, title VARCHAR(100), start_time TIME, event_date DATE
  /sql create table orders id INT PRIMARY KEY, amount FLOAT, customer VARCHAR(50)

  # ❌ Invalid schemas (will show helpful error messages)
  /sql create table users name VARCHAR          # Missing size
  /sql create table users id INT(11)           # INT doesn't support size
  /sql create table users active BOOLEAN(1)    # BOOLEAN doesn't support size
  ```

### Data Operations

- **Insert data into a table with schema validation:**

  - `/sql insert into table:users data:1, 'Alice Johnson', true`
  - Bot validates the data against the schema and stores it as:
    ```
    TIMESTAMP: 2025-08-19T00:14:00Z
    DATA:
      id: 1
      name: "Alice Johnson"
      active: true
    ```

- **Insert data into a flexible table:**
  - `/sql insert into table:logs data:'System started', '2025-08-19T00:14:00Z', 'INFO'`
  - Use SQL-style quoting for strings (single quotes). Flexible tables accept any values and are stored without schema validation.

### Data Querying with SELECT

SQLcord supports comprehensive SELECT queries with dynamic formatting and advanced WHERE conditions:

**Basic SELECT operations:**

- **Select all data:**

  - `/sql select columns:* from:users`
  - Returns all columns and rows from the users table

- **Select specific columns:**
  - `/sql select columns:name, email from:users`
  - Returns only the specified columns

**Advanced WHERE clauses with AND/OR logic:**

- **Single condition:**

  - `/sql select columns:* from:users where:name='John'`
  - Returns rows where name equals 'John'

- **AND logic (all conditions must be true):**

  - `/sql select columns:* from:users where:name='John' AND age=25`
  - Returns rows where both name is 'John' AND age is 25

- **OR logic (any condition can be true):**

  - `/sql select columns:* from:users where:name='John' OR name='Jane'`
  - Returns rows where name is either 'John' OR 'Jane'

- **Mixed AND/OR logic:**

  - `/sql select columns:* from:products where:category='Electronics' AND price=100 OR category='Books'`
  - Returns products that are either (Electronics AND price $100) OR Books

- **Parentheses for grouping:**
  - `/sql select columns:* from:users where:(name='John' OR name='Jane') AND age=25`
  - Returns users named John OR Jane who are also 25 years old
- **Complex nested conditions:**

  - `/sql select columns:* from:products where:(category='Electronics' OR category='Gaming') AND (price=100 OR price=200)`
  - Returns products in Electronics OR Gaming categories with price $100 OR $200

- **Advanced grouping:**
  - `/sql select columns:* from:employees where:(department='IT' AND role='Developer') OR (department='Sales' AND role='Manager')`
  - Returns IT Developers OR Sales Managers

**Additional SELECT features:**

- **DISTINCT filtering:**

  - `/sql select columns:category from:products distinct:true`
  - Returns unique values only, removing duplicates

- **Dynamic table formatting:**
  - Automatically adjusts column widths based on content
  - Handles long text values gracefully
  - Shows up to 20 rows with truncation indicators for larger results

**WHERE clause operator precedence:**

- **Parentheses** have highest precedence (force evaluation order)
- **AND** has higher precedence than OR
- **OR** has lowest precedence
- Example: `A AND B OR C` evaluates as `(A AND B) OR C`
- Use parentheses to override: `A AND (B OR C)` evaluates B OR C first
- Use proper spacing: `column='value' AND other='value'` (spaces around AND/OR)
- Complex example: `(A OR B) AND (C OR D)` ensures both groups are evaluated first

**SELECT result format:**

- Blue info-style embed with query statistics
- Formatted table with row numbers
- Query information (table, columns, filters, row count)
- Truncation notes for long values or large result sets

## Behavior ⚠️

- In-memory only: selections and metadata are not persisted. Restarting the bot clears state.
- Slash commands are registered globally by default in this repo - global registration can take up to ~1 hour to appear.
- The bot uses slash commands only; it does not parse message content (no privileged Message Content intent required).

## Notes 📚

- **Schema Persistence:** Table schemas are stored in Discord channel topics and persist across bot restarts
- **Data Storage:** Row data is stored as structured messages in table channels with timestamps
- **In-memory State:** User database selections are kept in memory only and reset on bot restart
- **Slash Commands:** The bot uses slash commands exclusively; no message content parsing required
- **Global Registration:** Slash commands are registered globally and may take up to 1 hour to appear initially
- **Development Status:** This is an experimental and educational project in active development

### Implementation Status

- ✔️ **Database creation** - Create and manage database categories
- ✔️ **Table creation** - Create tables with optional schema definitions
- ✔️ **Data insertion** - Insert validated data with schema support
- ✔️ **Schema validation** - Type checking and constraint validation
- ✔️ **Backward compatibility** - Support for legacy schema formats
- ✔️ **Data querying** - SELECT operations with full WHERE logic (AND/OR/parentheses)
- 🚧 **Data modification** - UPDATE and DELETE operations (stubs exist)
- 🚧 **Advanced features** - Joins, indexing, transactions (future work)

### Technical Details

- `src/commands/sql/use_.rs` is named `use_.rs` because `use` is reserved in Rust
- The `state/` folder contains the in-memory session store for per-user database selections
- Command registration happens at runtime during bot `ready` event
- All logging uses consistent formatting optimized for terminal compatibility
- Schema parsing supports standard SQL data types with size specifications

## Directory structure 📂

Below is a high-level view of this repository and a short explanation of important files and folders. Use this as a map when navigating the codebase. It is not guaranteed that the structure will stay this way.

```
src/
├─ main.rs                        // Entrypoint. Loads env, initializes logging, creates the Client via `bot::create_client_from_env()` and starts it.
├─ bot.rs                         // Builds `serenity::Client`, inserts shared TypeMap (CurrentDB), and provides `register_commands()` which builds the global `/sql` command. Registration is invoked at runtime from `handler::ready()`.
├─ handler.rs                     // `EventHandler` implementation: registers commands on ready and routes `Interaction::Command` events to the command handling logic.
├─ lib.rs                         // Library interface exposing modules for test binaries.
├─ logging.rs                     // Centralized logging system with consistent formatting and color codes. Used throughout the project instead of individual println! calls.
├─ sql_parser.rs                  // SQL parsing utilities for column definitions, data types, and value parsing with validation.
├─ utils.rs                       // Small helpers: IDs, sanitizers, embed creators for consistent user interface.
│
├─ bin/                           // Test binaries for development and verification.
│  ├─ test_registration.rs        // Tests dynamic command registration system.
│  ├─ test_run_functions.rs       // Tests all command run functions.
│  └─ test_bot_startup.rs         // Simulates bot startup with formatted logging.
│
├─ commands/                      // Command implementations grouped by domain.
│  ├─ mod.rs                      // Declares `sql` and `admin` modules. Contains `register_all_sql_commands()` for dynamic registration.
│  ├─ sql/
│  │  ├─ mod.rs                   // `/sql` group entrypoint and dynamic registration coordinator.
│  │  ├─ create/
│  │  │  ├─ mod.rs                // `/sql create` subcommands with `register_create_subcommands()`.
│  │  │  ├─ db.rs                 // `/sql create db` -> creates category `db_<name>`.
│  │  │  └─ table.rs              // `/sql create table` -> creates a channel under the selected `db_<name>` with optional schema stored in channel topic.
│  │  ├─ drop/
│  │  │  ├─ mod.rs                // `/sql drop` subcommands with `register_drop_subcommands()`.
│  │  │  ├─ db.rs                 // `/sql drop db` -> delete category (with safety checks).
│  │  │  └─ table.rs              // `/sql drop table` -> delete channel.
│  │  ├─ use_.rs                  // `/sql use <name>` -> set active DB for the user in this guild (stores in `CurrentDB`).
│  │  ├─ select.rs                // `/sql select ...` -> read messages from a channel and filter (implementation details vary).
│  │  ├─ insert.rs                // `/sql insert into ...` -> validate data against schema and send formatted message as a "row" into the target channel.
│  │  ├─ update.rs                // `/sql update ...` -> edit messages that match criteria (where clauses are simple/stubbed).
│  │  ├─ delete.rs                // `/sql delete ...` -> delete messages that match criteria.
│  │  └─ explain.rs               // `/sql explain ...` -> describe the Discord operations that will be performed.
│  │
│  └─ admin/                      // Non-SQL bot admin commands and helpers.
│     ├─ mod.rs
│     ├─ perms.rs
│     └─ debug.rs
│
├─ services/                      // Centralizes Discord API calls and permission-aware helpers (`discord_fs.rs`) and encoding helpers (`encode.rs`).
│  ├─ mod.rs
│  ├─ discord_fs.rs
│  └─ encode.rs
│
├─ state/                         // In-memory per-guild per-user session state.
│  ├─ mod.rs
│  └─ session_store.rs            // Defines `CurrentDB` as `Arc<Mutex<HashMap<(GuildId, UserId), String>>>`.
│
├─ guards/                        // Validation and policy checks used by command handlers (`safety.rs`).
│  ├─ mod.rs
│  └─ safety.rs
│
├─ render/                        // Reply helpers for consistent user-facing messages (embeds, tables).
│  ├─ mod.rs
│  ├─ reply.rs
│  └─ table.rs
│
└─ utils.rs                       // Small helpers: IDs, tiny parsers, sanitizers.
```
