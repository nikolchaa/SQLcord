mod handler;
mod bot;
mod commands;
mod state;
mod guards;
mod render;
mod services;
mod utils;
mod logging;
mod sql_parser;

use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    // load .env
    dotenv().ok();

    tracing_subscriber::fmt::init();

    let mut client = match bot::create_client_from_env().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to create client: {e}");
            return;
        }
    };

    if let Err(e) = client.start().await {
        tracing::error!("Client error: {e}");
    }
}
