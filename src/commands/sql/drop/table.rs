// /sql drop table <name>

use std::error::Error;
use serenity::prelude::Context;
use serenity::model::id::{GuildId, UserId};
use serenity::model::channel::ChannelType;
use crate::state::CurrentDB;
use crate::logging::log_info;
use crate::utils::sanitize_channel_name;

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering DROP TABLE command");
    Ok(())
}

/// Attempt to drop the table channel named `table_<table_name>` from the current database.
/// Returns Ok(success_message) or Err(error_message).
pub async fn run(ctx: &Context, guild_id: GuildId, user_id: UserId, table_name: &str) -> Result<String, String> {
    log_info(&format!("DROP TABLE command executed for table: {}", table_name));
    
    // Sanitize the table name
    let (sanitized_name, was_changed) = sanitize_channel_name(table_name);
    
    if sanitized_name.is_empty() {
        return Err("Table name cannot be empty after sanitization.".to_string());
    }
    
    // Get the current database for this user
    let data_read = ctx.data.read().await;
    let current_db = if let Some(map_arc) = data_read.get::<CurrentDB>() {
        let map = map_arc.lock().await;
        map.get(&(guild_id, user_id)).cloned()
    } else {
        None
    };
    drop(data_read);

    let current_db = match current_db {
        Some(db_name) => db_name,
        None => return Err("No database selected. Use `/sql use <db_name>` first.".to_string()),
    };

    // Find the database category and table channel
    match guild_id.channels(&ctx.http).await {
        Ok(channels) => {
            let db_category_name = format!("db_{}", current_db);
            let db_category = channels.values()
                .find(|c| c.name == db_category_name && c.kind == ChannelType::Category);
            
            if let Some(category) = db_category {
                // Find the table channel
                let table_channel_name = format!("table_{}", sanitized_name);
                let table_channel = channels.values()
                    .find(|c| c.name == table_channel_name && c.parent_id == Some(category.id));
                
                if let Some(table) = table_channel {
                    match table.id.delete(&ctx.http).await {
                        Ok(_) => {
                            let mut success_msg = format!("Table `{}` deleted from database `{}`", sanitized_name, current_db);
                            if was_changed {
                                success_msg.push_str(&format!(" (name sanitized from `{}` to `{}`)", table_name, sanitized_name));
                            }
                            log_info(&format!("SUCCESS: {}", success_msg));
                            Ok(success_msg)
                        },
                        Err(e) => {
                            tracing::error!("Failed to delete table channel: {e}");
                            let error_msg = "Failed to delete table. Check bot permissions.".to_string();
                            log_info(&format!("ERROR: {}", error_msg));
                            Err(error_msg)
                        }
                    }
                } else {
                    Err(format!("Table `{}` not found in database `{}`", sanitized_name, current_db))
                }
            } else {
                Err(format!("Database `{}` not found. Use `/sql use <db_name>` to select a database first.", current_db))
            }
        },
        Err(e) => {
            tracing::error!("Failed to get channels: {e}");
            Err("Failed to list channels. Check bot permissions.".to_string())
        }
    }
}
