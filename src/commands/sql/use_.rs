// /sql use <name>

use std::error::Error;
use serenity::prelude::Context;
use serenity::model::id::{GuildId, UserId};
use serenity::model::channel::ChannelType;
use crate::state::CurrentDB;
use crate::logging::log_info;
use crate::utils::sanitize_channel_name;

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering USE command");
    Ok(())
}

/// Set the current DB for a user in a guild. Returns Ok(success_message) or Err(error_message).
pub async fn run(ctx: &Context, guild_id: GuildId, user_id: UserId, db_name: &str) -> Result<String, String> {
    log_info(&format!("USE command executed for database: {} by user: {}", db_name, user_id));
    
    // Sanitize the database name
    let (sanitized_name, was_changed) = sanitize_channel_name(db_name);
    
    if sanitized_name.is_empty() {
        return Err("Database name cannot be empty after sanitization.".to_string());
    }
    
    // Verify the database exists
    let db_category_name = format!("db_{}", sanitized_name);
    match guild_id.channels(&ctx.http).await {
        Ok(channels) => {
            let db_exists = channels.values()
                .any(|c| c.name == db_category_name && c.kind == ChannelType::Category);
            
            if !db_exists {
                return Err(format!("Database `{}` not found. Create it first with `/sql create db {}`", 
                    db_category_name, sanitized_name));
            }
        },
        Err(e) => {
            tracing::error!("Failed to get channels: {e}");
            return Err("Failed to list channels. Check bot permissions.".to_string());
        }
    }
    
    let data_read = ctx.data.read().await;
    if let Some(map_arc) = data_read.get::<CurrentDB>().cloned() {
        drop(data_read);
        let mut map = map_arc.lock().await;
        map.insert((guild_id, user_id), sanitized_name.clone());
        
        let mut success_msg = format!("Using database `{}`", db_category_name);
        if was_changed {
            success_msg.push_str(&format!(" (name sanitized from `{}` to `{}`)", db_name, sanitized_name));
        }
        Ok(success_msg)
    } else {
        Err("Internal error: data map missing".to_string())
    }
}
