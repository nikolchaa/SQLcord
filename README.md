# 🥏 SQLcord - SQL prompts inside Discord (Rust)

A Discord bot for executing SQL commands in a fun and educational way. This project maps:

- Databases -> Discord categories (named `db_<name>`)
- Tables -> Channels inside the category
- Rows -> Messages inside the table channel (future work)

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

### Table Schema Support

SQLcord supports defining table schemas when creating tables:

```bash
/sql create table users id INT, name VARCHAR(255), active BOOLEAN
```

**Supported Data Types:**

- `INT`, `INTEGER` - Integer numbers
- `VARCHAR(size)`, `CHAR(size)` - Text with optional size limit
- `BOOLEAN`, `BOOL` - True/false values
- `FLOAT`, `DOUBLE`, `DECIMAL` - Decimal numbers
- `DATE`, `TIME`, `DATETIME` - Date and time values

**Schema Features:**

- **Type validation** - INSERT commands validate data against the defined schema
- **Flexible insertion** - Tables without schemas accept any data format
- **Backward compatibility** - Automatically handles tables created with older schema formats
- **Storage format** - Schemas are stored in Discord channel topics for persistence

**Example INSERT with schema validation:**

```bash
/sql insert into users 1, 'John Doe', true
```

Results in structured data storage:

```
TIMESTAMP: 2025-08-19 00:14:00 UTC
DATA:
  id: 1
  name: "John Doe"
  active: true
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

## Examples ✨

### Basic Database Operations

- **Create a database called `test`:**
  - Use the slash command: `/sql create db name:test`
  - Bot replies: "Database `db_test` created"
- **Use the database you just created:**
  - `/sql use name:test`
  - Bot replies: "Using database `db_test`"

### Table Operations with Schema

- **Create a table with schema:**
  - `/sql create table name:users schema:id INT, name VARCHAR(255), active BOOLEAN`
  - Bot creates a channel `table_users` with the defined schema
- **Create a simple table without schema:**
  - `/sql create table name:logs`
  - Bot creates a flexible table that accepts any data format

### Data Operations

- **Insert data into a table with schema validation:**

  - `/sql insert into table:users data:1, 'Alice Johnson', true`
  - Bot validates the data against the schema and stores it as:
    ```
    TIMESTAMP: 2025-08-19 00:14:00 UTC
    DATA:
      id: 1
      name: "Alice Johnson"
      active: true
    ```

- **Insert data into a flexible table:**
  - `/sql insert into table:logs data:'System started', 2025-08-19, INFO`
  - Bot stores the data without schema validation

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

- ✅ **Database creation** - Create and manage database categories
- ✅ **Table creation** - Create tables with optional schema definitions
- ✅ **Data insertion** - Insert validated data with schema support
- ✅ **Schema validation** - Type checking and constraint validation
- ✅ **Backward compatibility** - Support for legacy schema formats
- 🚧 **Data querying** - SELECT operations (basic implementation exists)
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
