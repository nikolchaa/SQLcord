# 🥏 SQLcord - SQL-like interface inside Discord (Rust)

Bring SQL-ish workflows into Discord channels and categories. This project maps:

- Databases -> Discord categories (named `db_<name>`)
- Tables -> Channels inside the category
- Rows -> Messages inside the table channel (future work)

Quick start 🚀

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

Commands implemented 🛠️

- `/sql create db <name>` - creates a category named `db_<name>`.
- `/sql use <name>` - selects an existing `db_<name>` for your user (kept in-memory per guild+user).

Examples ✨

- Create a database called `test`:
  - Use the slash command: `/sql create db name:test`
  - Bot replies: "Database `db_test` created"
- Use the database you just created:
  - `/sql use name:test`
  - Bot replies: "Using database `db_test`"

Notes & behavior ⚠️

- In-memory only: selections and metadata are not persisted. Restarting the bot clears state.
- Slash commands are registered globally by default in this repo - global registration can take up to ~1 hour to appear.
- The bot uses slash commands only; it does not parse message content (no privileged Message Content intent required).

Notes 📚

- This is intended as an experimental, educational, and very much MEME project.
