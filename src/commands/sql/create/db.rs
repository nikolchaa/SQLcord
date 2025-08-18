// /sql create db <name>

use std::error::Error;
use serenity::prelude::Context;
use serenity::model::id::GuildId;
use serenity::model::channel::ChannelType;
use crate::logging::log_info;
use crate::utils::{sanitize_channel_name, create_success_embed, create_error_embed};

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering CREATE DB command");
    Ok(())
}

/// Create a category named `db_<db_name>` in the given guild.
/// Returns Ok(embed) or Err(embed).
pub async fn run(ctx: &Context, guild_id: GuildId, db_name: &str) -> Result<serenity::builder::CreateEmbed, serenity::builder::CreateEmbed> {
    log_info(&format!("CREATE DB command executed for database: {}", db_name));
    
    // Sanitize the database name
    let (sanitized_name, was_changed) = sanitize_channel_name(db_name);
    
    if sanitized_name.is_empty() {
        let embed = create_error_embed(
            "❌ Invalid Database Name",
            "Database name cannot be empty after sanitization. Please provide a valid name with alphanumeric characters."
        );
        return Err(embed);
    }
    
    let channel_name = format!("db_{}", sanitized_name);
    let builder = serenity::builder::CreateChannel::new(&channel_name).kind(ChannelType::Category);
    
    match guild_id.create_channel(&ctx.http, builder).await {
        Ok(_) => {
            let mut description = format!("Database **{}** has been created successfully!", channel_name);
            if was_changed {
                description.push_str(&format!("\n\n*Name sanitized from `{}` to `{}`*", db_name, sanitized_name));
            }
            
            let embed = create_success_embed("✅ Database Created", &description);
            log_info(&format!("SUCCESS: Database {} created", channel_name));
            Ok(embed)
        },
        Err(e) => {
            tracing::error!("Failed to create category: {e}");
            let embed = create_error_embed(
                "❌ Database Creation Failed",
                "Failed to create database. Please check bot permissions or try again."
            );
            log_info("ERROR: Failed to create database");
            Err(embed)
        }
    }
}
