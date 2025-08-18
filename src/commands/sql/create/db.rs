// /sql create db <name>

use std::error::Error;
use serenity::prelude::Context;
use serenity::model::id::GuildId;
use serenity::model::channel::ChannelType;
use crate::logging::log_info;
use crate::utils::sanitize_channel_name;

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering CREATE DB command");
    Ok(())
}

/// Create a category named `db_<db_name>` in the given guild.
/// Returns Ok(success_message) or Err(error_message).
pub async fn run(ctx: &Context, guild_id: GuildId, db_name: &str) -> Result<String, String> {
    log_info(&format!("CREATE DB command executed for database: {}", db_name));
    
    // Sanitize the database name
    let (sanitized_name, was_changed) = sanitize_channel_name(db_name);
    
    if sanitized_name.is_empty() {
        return Err("Database name cannot be empty after sanitization.".to_string());
    }
    
    let channel_name = format!("db_{}", sanitized_name);
    let builder = serenity::builder::CreateChannel::new(&channel_name).kind(ChannelType::Category);
    
    match guild_id.create_channel(&ctx.http, builder).await {
        Ok(_) => {
            let mut success_msg = format!("Database `{}` created", channel_name);
            if was_changed {
                success_msg.push_str(&format!(" (name sanitized from `{}` to `{}`)", db_name, sanitized_name));
            }
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
