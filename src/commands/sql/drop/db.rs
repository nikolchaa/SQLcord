// /sql drop db <name>

use std::error::Error;
use serenity::prelude::Context;
use serenity::model::id::GuildId;
use serenity::model::channel::ChannelType;
use crate::logging::{log_info, log_error};
use crate::utils::{sanitize_channel_name, create_success_embed, create_error_embed, create_warning_embed};

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering DROP DB command");
    Ok(())
}

/// Attempt to drop the category named `db_<db_name>` in the guild.
/// Returns Ok(embed) or Err(embed).
pub async fn run(ctx: &Context, guild_id: GuildId, db_name: &str) -> Result<serenity::builder::CreateEmbed, serenity::builder::CreateEmbed> {
    log_info(&format!("DROP DB command executed for database: {}", db_name));
    
    // Sanitize the database name
    let (sanitized_name, was_changed) = sanitize_channel_name(db_name);
    
    if sanitized_name.is_empty() {
        let embed = create_error_embed(
            "❌ Invalid Database Name",
            "Database name cannot be empty after sanitization. Please provide a valid name."
        );
        return Err(embed);
    }
    
    match guild_id.channels(&ctx.http).await {
        Ok(chans) => {
            let target = format!("db_{}", sanitized_name);
            let found = chans.values().find(|c| c.name == target && c.kind == ChannelType::Category);
            if let Some(cat) = found {
                // ensure category has no child channels
                let child_count = chans.values().filter(|c| c.parent_id == Some(cat.id)).count();
                if child_count > 0 {
                    let embed = create_warning_embed(
                        "⚠️ Cannot Drop Database",
                        &format!("Refusing to drop **{}**: category is not empty ({} tables). Remove all tables first.", target, child_count)
                    );
                    Err(embed)
                } else {
                    match cat.id.delete(&ctx.http).await {
                        Ok(_) => {
                            let mut description = format!("Database **{}** has been deleted successfully!", target);
                            if was_changed {
                                description.push_str(&format!("\n\n*Name sanitized from `{}` to `{}`*", db_name, sanitized_name));
                            }
                            let embed = create_success_embed("✅ Database Deleted", &description);
                            Ok(embed)
                        },
                        Err(e) => {
                            tracing::error!("Failed to delete category: {e}");
                            let embed = create_error_embed(
                                "❌ Database Deletion Failed",
                                "Failed to delete database. Please check bot permissions or try again."
                            );
                            log_error(&format!("{}", e));
                            Err(embed)
                        }
                    }
                }
            } else {
                let embed = create_error_embed(
                    "❌ Database Not Found",
                    &format!("Database **{}** was not found in this server.", target)
                );
                Err(embed)
            }
        },
        Err(e) => {
            tracing::error!("Failed to get channels: {e}");
            let embed = create_error_embed(
                "❌ Permission Error",
                "Failed to list channels. Please check bot permissions."
            );
            Err(embed)
        }
    }
}
