// /sql use <name>

use std::error::Error;
use serenity::prelude::Context;
use serenity::model::id::{GuildId, UserId};
use crate::state::CurrentDB;
use crate::logging::log_info;

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering USE command");
    Ok(())
}

/// Set the current DB for a user in a guild. Returns Ok(success_message) or Err(error_message).
pub async fn run(ctx: &Context, guild_id: GuildId, user_id: UserId, db_name: &str) -> Result<String, String> {
    log_info(&format!("USE command executed for database: {} by user: {}", db_name, user_id));
    let data_read = ctx.data.read().await;
	if let Some(map_arc) = data_read.get::<CurrentDB>().cloned() {
		drop(data_read);
		let mut map = map_arc.lock().await;
		map.insert((guild_id, user_id), db_name.to_string());
		Ok(format!("Using database `db_{}`", db_name))
	} else {
		Err("Internal error: data map missing".to_string())
	}
}
