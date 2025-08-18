mod handler;
mod state;

use dotenvy::dotenv;
use serenity::prelude::*;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use state::CurrentDB;

use handler::Handler;



#[tokio::main]
async fn main() {
    // load .env
    dotenv().ok();

    tracing_subscriber::fmt::init();

    let token = match env::var("DISCORD_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            tracing::error!("DISCORD_TOKEN not set in environment or .env file");
            return;
        }
    };

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = match Client::builder(&token, intents).event_handler(Handler).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to create client: {e}");
            return;
        }
    };

    // initialize shared data: CurrentDB map
    {
        let mut data = client.data.write().await;
        data.insert::<CurrentDB>(Arc::new(Mutex::new(HashMap::new())));
    }

    if let Err(e) = client.start().await {
        tracing::error!("Client error: {e}");
    }
}
