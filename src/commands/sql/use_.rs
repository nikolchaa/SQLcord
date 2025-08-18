// /sql use <name>

use std::error::Error;
use serenity::prelude::Context;
use serenity::model::id::{GuildId, UserId};
use serenity::model::channel::ChannelType;
use crate::state::CurrentDB;
use crate::logging::log_info;
use crate::utils::{sanitize_channel_name, create_success_embed, create_error_embed};

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering USE command");
    Ok(())
}

/// Set the current DB for a user in a guild. Returns Ok(embed) or Err(embed).
pub async fn run(ctx: &Context, guild_id: GuildId, user_id: UserId, db_name: &str) -> Result<serenity::builder::CreateEmbed, serenity::builder::CreateEmbed> {
    log_info(&format!("USE command executed for database: {} by user: {}", db_name, user_id));
    
    // Sanitize the database name
    let (sanitized_name, was_changed) = sanitize_channel_name(db_name);
    
    if sanitized_name.is_empty() {
        let embed = create_error_embed(
            "❌ Invalid Database Name",
            "Database name cannot be empty after sanitization. Please provide a valid name."
        );
        return Err(embed);
    }
    
    // Verify the database exists
    let db_category_name = format!("db_{}", sanitized_name);
    match guild_id.channels(&ctx.http).await {
        Ok(channels) => {
            let db_exists = channels.values()
                .any(|c| c.name == db_category_name && c.kind == ChannelType::Category);
            
            if !db_exists {
                let embed = create_error_embed(
                    "❌ Database Not Found",
                    &format!("Database **{}** was not found. Create it first with `/sql create db {}`", db_category_name, sanitized_name)
                );
                return Err(embed);
            }
        },
        Err(e) => {
            tracing::error!("Failed to get channels: {e}");
            let embed = create_error_embed(
                "❌ Permission Error",
                "Failed to list channels. Please check bot permissions."
            );
            return Err(embed);
        }
    }
    
    let data_read = ctx.data.read().await;
    if let Some(map_arc) = data_read.get::<CurrentDB>().cloned() {
        drop(data_read);
        let mut map = map_arc.lock().await;
        map.insert((guild_id, user_id), sanitized_name.clone());
        
        let mut description = format!("Now using database **{}**", db_category_name);
        if was_changed {
            description.push_str(&format!("\n\n*Name sanitized from `{}` to `{}`*", db_name, sanitized_name));
        }
        let embed = create_success_embed("✅ Database Selected", &description);
        Ok(embed)
    } else {
        let embed = create_error_embed(
            "❌ Internal Error",
            "Data map missing. Please try again or contact support."
        );
        Err(embed)
    }
}
