// /sql create db <name>

use std::error::Error;
use serenity::prelude::Context;
use serenity::model::id::GuildId;
use serenity::model::channel::ChannelType;
use crate::logging::log_info;

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering CREATE DB command");
    Ok(())
}

/// Create a category named `db_<db_name>` in the given guild.
/// Returns Ok(success_message) or Err(error_message).
pub async fn run(ctx: &Context, guild_id: GuildId, db_name: &str) -> Result<String, String> {
    log_info(&format!("CREATE DB command executed for database: {}", db_name));
    let builder = serenity::builder::CreateChannel::new(format!("db_{}", db_name)).kind(ChannelType::Category);
    match guild_id.create_channel(&ctx.http, builder).await {
        Ok(_) => {
            let success_msg = format!("Database `db_{}` created", db_name);
            log_info(&format!("SUCCESS: {}", success_msg));
            Ok(success_msg)
        },
        Err(e) => {
            tracing::error!("Failed to create category: {e}");
            let error_msg = "Failed to create database. Check bot permissions.".to_string();
            log_info(&format!("ERROR: {}", error_msg));
            Err(error_msg)
        }
    }
}
