// /sql drop db <name>

use std::error::Error;
use serenity::prelude::Context;
use serenity::model::id::GuildId;
use serenity::model::channel::ChannelType;
use crate::logging::log_info;
use crate::utils::sanitize_channel_name;

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering DROP DB command");
    Ok(())
}

/// Attempt to drop the category named `db_<db_name>` in the guild.
/// Returns Ok(success_message) or Err(error_message).
pub async fn run(ctx: &Context, guild_id: GuildId, db_name: &str) -> Result<String, String> {
    log_info(&format!("DROP DB command executed for database: {}", db_name));
    
    // Sanitize the database name
    let (sanitized_name, was_changed) = sanitize_channel_name(db_name);
    
    if sanitized_name.is_empty() {
        return Err("Database name cannot be empty after sanitization.".to_string());
    }
    
    match guild_id.channels(&ctx.http).await {
        Ok(chans) => {
            let target = format!("db_{}", sanitized_name);
            let found = chans.values().find(|c| c.name == target && c.kind == ChannelType::Category);
            if let Some(cat) = found {
                // ensure category has no child channels
                let child_count = chans.values().filter(|c| c.parent_id == Some(cat.id)).count();
                if child_count > 0 {
                    Err(format!("Refusing to drop `{}`: category is not empty ({} channels). Remove tables first.", target, child_count))
                } else {
                    match cat.id.delete(&ctx.http).await {
                        Ok(_) => {
                            let mut success_msg = format!("Database `{}` deleted", target);
                            if was_changed {
                                success_msg.push_str(&format!(" (name sanitized from `{}` to `{}`)", db_name, sanitized_name));
                            }
                            Ok(success_msg)
                        },
                        Err(e) => {
                            tracing::error!("Failed to delete category: {e}");
                            Err("Failed to delete database. Check bot permissions.".to_string())
                        }
                    }
                }
            } else {
                Err(format!("Database `{}` not found", target))
            }
        },
        Err(e) => {
            tracing::error!("Failed to get channels: {e}");
            Err("Failed to list channels. Check bot permissions.".to_string())
        }
    }
}
