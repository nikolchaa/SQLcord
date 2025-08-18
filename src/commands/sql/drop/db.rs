// /sql drop db <name>

use std::error::Error;
use serenity::prelude::Context;
use serenity::model::id::GuildId;
use serenity::model::channel::ChannelType;
use crate::logging::log_info;

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering DROP DB command");
    Ok(())
}

/// Attempt to drop the category named `db_<db_name>` in the guild.
/// Returns Ok(success_message) or Err(error_message).
pub async fn run(ctx: &Context, guild_id: GuildId, db_name: &str) -> Result<String, String> {
    log_info(&format!("DROP DB command executed for database: {}", db_name));
    match guild_id.channels(&ctx.http).await {
		Ok(chans) => {
			let target = format!("db_{}", db_name);
			let found = chans.values().find(|c| c.name == target && c.kind == ChannelType::Category);
			if let Some(cat) = found {
				// ensure category has no child channels
				let child_count = chans.values().filter(|c| c.parent_id == Some(cat.id)).count();
				if child_count > 0 {
					Err(format!("Refusing to drop `db_{}`: category is not empty ({} channels). Remove tables first.", db_name, child_count))
				} else {
					match cat.id.delete(&ctx.http).await {
						Ok(_) => Ok(format!("Database `db_{}` deleted", db_name)),
						Err(e) => {
							tracing::error!("Failed to delete category: {e}");
							Err("Failed to delete database. Check bot permissions.".to_string())
						}
					}
				}
			} else {
				Err(format!("Database `db_{}` not found", db_name))
			}
		},
		Err(e) => {
			tracing::error!("Failed to get channels: {e}");
			Err("Failed to list channels. Check bot permissions.".to_string())
		}
	}
}
