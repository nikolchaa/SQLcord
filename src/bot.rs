use std::env;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use serenity::Client;
use serenity::model::gateway::GatewayIntents;
use crate::state::CurrentDB;
use crate::handler::Handler;

pub async fn create_client_from_env() -> Result<Client, Box<dyn std::error::Error>> {
    let token = env::var("DISCORD_TOKEN")?;

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let client = Client::builder(&token, intents).event_handler(Handler).await?;

    // initialize shared data: CurrentDB map
    {
        let mut data = client.data.write().await;
        data.insert::<CurrentDB>(Arc::new(Mutex::new(HashMap::new())));
    }

    // command registration is performed after the client is ready (in handler.rs)

    Ok(client)
}

pub async fn register_commands(http: &serenity::http::Http) -> Result<(), Box<dyn std::error::Error>> {
    use serenity::builder::CreateCommand;
    use std::time::Duration;

    // Ensure application info is available (some environments populate it lazily).
    // Retry a few times with a short backoff before proceeding.
    let mut attempts = 0u8;
    loop {
        match http.get_current_application_info().await {
            Ok(_) => break,
            Err(e) => {
                attempts += 1;
                if attempts >= 5 {
                    return Err(Box::new(e));
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }

    // Build command(s) centrally from src/commands
    let builders: Vec<CreateCommand> = crate::commands::register_all();

    for builder in builders {
        serenity::model::application::Command::create_global_command(http, builder).await?;
    }

    // Ensure application info is fetched (some serenity versions require this to populate app id)
    let _ = http.get_current_application_info().await;

    Ok(())
}
