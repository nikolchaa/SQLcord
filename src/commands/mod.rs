// Top-level commands module
pub mod sql;
pub mod admin;
use serenity::builder::CreateCommand;

/// Build and return the top-level command builders for registration.
/// Currently returns the `/sql` command CreateCommand builder.
pub fn register_all() -> Vec<CreateCommand> {
    let mut v = Vec::new();
    v.push(sql::register_sql_group());
    // admin command group (may be empty/placeholder)
    v.push(admin::register_admin_group());
    v
}
