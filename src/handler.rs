use crate::state::CurrentDB;
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::application::Interaction;
use serenity::model::application::CommandDataOptionValue;
use serenity::model::channel::ChannelType;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage, CreateChannel};
use serenity::prelude::*;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        tracing::info!("{} is connected!", ready.user.name);
        // register global commands now that we're ready
        if let Err(e) = crate::bot::register_commands(&_ctx.http).await {
            tracing::error!("Failed to create sql command: {e}");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            match command.data.name.as_str() {
                "sql" => {
                    // options may contain a subcommand group (create) and/or subcommands (use). Iterate to find which was used.
                    for opt in &command.data.options {
                        match opt.name.as_str() {
                            "create" => {
                                match &opt.value {
                                    CommandDataOptionValue::SubCommandGroup(groups) => {
                                        if let Some(sub) = groups.get(0) {
                                            if sub.name == "db" {
                                                if let CommandDataOptionValue::SubCommand(params) = &sub.value {
                                                    if let Some(name_opt) = params.get(0) {
                                                        if let CommandDataOptionValue::String(db_name) = &name_opt.value {
                                                            if let Some(guild_id) = command.guild_id {
                                                                let builder = CreateChannel::new(format!("db_{}", db_name)).kind(ChannelType::Category);
                                                                let created = guild_id.create_channel(&ctx.http, builder).await;

                                                                match created {
                                                                    Ok(_) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(format!("Database `db_{}` created", db_name))
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after creating db: {e}");
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        tracing::error!("Failed to create category: {e}");
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content("Failed to create database. Check bot permissions.")
                                                                        )).await {
                                                                            tracing::error!("Failed to send error response: {e}");
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                    CreateInteractionResponseMessage::new().content("This command must be used in a server (guild).")
                                                                )).await {
                                                                    tracing::error!("Failed to send guild-only response: {e}");
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    CommandDataOptionValue::SubCommand(params) => {
                                        // handle if create was registered as subcommand directly
                                        if let Some(sub) = params.get(0) {
                                            if sub.name == "db" {
                                                if let CommandDataOptionValue::SubCommand(inner) = &sub.value {
                                                    if let Some(name_opt) = inner.get(0) {
                                                        if let CommandDataOptionValue::String(db_name) = &name_opt.value {
                                                            if let Some(guild_id) = command.guild_id {
                                                                let _ = guild_id.create_channel(&ctx.http, CreateChannel::new(format!("db_{}", db_name)).kind(ChannelType::Category)).await;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            "use" => {
                                match &opt.value {
                                    CommandDataOptionValue::SubCommand(params) => {
                                        if let Some(name_opt) = params.get(0) {
                                            if let CommandDataOptionValue::String(db_name) = &name_opt.value {
                                                if let Some(guild_id) = command.guild_id {
                                                    match guild_id.channels(&ctx.http).await {
                                                        Ok(chans) => {
                                                            let target = format!("db_{}", db_name);
                                                            let found = chans.values().find(|c| c.name == target && c.kind == ChannelType::Category);
                                                            if let Some(_) = found {
                                                                let user_id = command.user.id;
                                                                let data_read = ctx.data.read().await;
                                                                if let Some(map_arc) = data_read.get::<CurrentDB>().cloned() {
                                                                    drop(data_read);
                                                                    let mut map = map_arc.lock().await;
                                                                    map.insert((guild_id, user_id), db_name.to_string());

                                                                    if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                        CreateInteractionResponseMessage::new().content(format!("Using database `db_{}`", db_name))
                                                                    )).await {
                                                                        tracing::error!("Failed to respond after setting current db: {e}");
                                                                    }
                                                                } else {
                                                                    if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                        CreateInteractionResponseMessage::new().content("Internal error: data map missing")
                                                                    )).await {
                                                                        tracing::error!("Failed to send internal error response: {e}");
                                                                    }
                                                                }
                                                            } else {
                                                                if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                    CreateInteractionResponseMessage::new().content(format!("Database `db_{}` not found", db_name))
                                                                )).await {
                                                                    tracing::error!("Failed to respond for missing db: {e}");
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            tracing::error!("Failed to fetch channels: {e}");
                                                            if let Err(e2) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                CreateInteractionResponseMessage::new().content("Failed to check databases. Check permissions.")
                                                            )).await {
                                                                tracing::error!("Failed to send channels error response: {e2}");
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                        CreateInteractionResponseMessage::new().content("This command must be used in a server (guild).")
                                                    )).await {
                                                        tracing::error!("Failed to send guild-only response: {e}");
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {
                    if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content("Unknown command")
                    )).await {
                        tracing::error!("Failed to respond to unknown command: {e}");
                    }
                }
            }
        }
    }
}
