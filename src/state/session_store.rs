use serenity::prelude::TypeMapKey;
use serenity::model::id::{GuildId, UserId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CurrentDB;

impl TypeMapKey for CurrentDB {
    type Value = Arc<Mutex<HashMap<(GuildId, UserId), String>>>;
}
