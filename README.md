# 🥏 SQLcord - SQL prompts inside Discord (Rust)

Play around with SQL inside Discord. This project maps:

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

## Commands implemented 🛠️

- `/sql create db <name>` - creates a category named `db_<name>`.
- `/sql use <name>` - selects an existing `db_<name>` for your user (kept in-memory per guild+user).

## Examples ✨

- Create a database called `test`:
  - Use the slash command: `/sql create db name:test`
  - Bot replies: "Database `db_test` created"
- Use the database you just created:
  - `/sql use name:test`
  - Bot replies: "Using database `db_test`"

## Behavior ⚠️

- In-memory only: selections and metadata are not persisted. Restarting the bot clears state.
- Slash commands are registered globally by default in this repo - global registration can take up to ~1 hour to appear.
- The bot uses slash commands only; it does not parse message content (no privileged Message Content intent required).

## Notes 📚

- This is intended as an experimental, educational, and very much MEME project.
- Most of the functionality is not there. This is in very early stages of development.
- `src/commands/sql/use_.rs` is named `use_.rs` because `use` is reserved in Rust.
- The `state/` folder contains the in-memory session store used to track per-user selected databases (not persisted).
- Command registration is performed at runtime when the bot becomes `ready` (in `handler::ready()`), so global commands may take time to appear after first run.

## Directory structure 📂

Below is a high-level view of this repository and a short explanation of important files and folders. Use this as a map when navigating the codebase. It is not guaranteed that the structure will stay this way.

```
src/
├─ main.rs                        // Entrypoint. Loads env, initializes logging, creates the Client via `bot::create_client_from_env()` and starts it.
├─ bot.rs                         // Builds `serenity::Client`, inserts shared TypeMap (CurrentDB), and provides `register_commands()` which builds the global `/sql` command. Registration is invoked at runtime from `handler::ready()`.
├─ handler.rs                     // `EventHandler` implementation: registers commands on ready and routes `Interaction::Command` events to the command handling logic.
│
├─ commands/                      // Command implementations grouped by domain.
│  ├─ mod.rs                      // Declares `sql` and `admin` modules. Currently contains a `register_all()` placeholder.
│  ├─ sql/
│  │  ├─ mod.rs                   // `/sql` group entrypoint and helpers.
│  │  ├─ create/
│  │  │  ├─ mod.rs                // `/sql create` subcommands.
│  │  │  ├─ db.rs                 // `/sql create db` -> creates category `db_<name>`.
│  │  │  └─ table.rs              // `/sql create table` -> intended to create a channel under the selected `db_<name>`.
│  │  ├─ drop/
│  │  │  ├─ mod.rs                // `/sql drop` subcommands.
│  │  │  ├─ db.rs                 // `/sql drop db` -> delete category (with safety checks).
│  │  │  └─ table.rs              // `/sql drop table` -> delete channel.
│  │  ├─ use_.rs                  // `/sql use <name>` -> set active DB for the user in this guild (stores in `CurrentDB`).
│  │  ├─ select.rs                // `/sql select ...` -> read messages from a channel and filter (implementation details vary).
│  │  ├─ insert.rs                // `/sql insert ...` -> send a message as a "row" into the target channel.
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
