use crate::state::CurrentDB;
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::prelude::interaction::Interaction;
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::channel::ChannelType;
use serenity::prelude::*;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("{} is connected!", ready.user.name);

        // Register /sql with create and use
        if let Err(e) = serenity::model::prelude::command::Command::create_global_application_command(&ctx.http, |c| {
            c.name("sql").description("Run SQL-like operations mapped to Discord")
            // create group: /sql create db <name>
            .create_option(|opt| {
                opt.name("create").kind(CommandOptionType::SubCommandGroup).description("Create resources").create_sub_option(|sub| {
                    sub.name("db").kind(CommandOptionType::SubCommand).description("Create a database (category)").create_sub_option(|s| {
                        s.name("name").kind(CommandOptionType::String).description("Database name").required(true)
                    })
                })
            })
            // use subcommand: /sql use <name>
            .create_option(|opt| {
                opt.name("use").kind(CommandOptionType::SubCommand).description("Select database to use").create_sub_option(|s| {
                    s.name("name").kind(CommandOptionType::String).description("Database name").required(true)
                })
            })
        }).await
        {
            tracing::error!("Failed to create sql command: {e}");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "sql" => {
                    // options may contain a subcommand group (create) and/or subcommands (use). Iterate to find which was used.
                    for opt in &command.data.options {
                        match opt.name.as_str() {
                            "create" => {
                                if let Some(sub) = opt.options.get(0) {
                                    if sub.name == "db" {
                                        if let Some(name_opt) = sub.options.get(0) {
                                            if let Some(val) = &name_opt.value {
                                                if let Some(db_name) = val.as_str() {
                                                    if let Some(guild_id) = command.guild_id {
                                                        let created = guild_id.create_channel(&ctx.http, |c| {
                                                            c.name(format!("db_{}", db_name)).kind(ChannelType::Category)
                                                        }).await;

                                                        match created {
                                                            Ok(_) => {
                                                                if let Err(e) = command.create_interaction_response(&ctx.http, |resp| {
                                                                    resp.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|m| m.content(format!("Database `db_{}` created", db_name)))
                                                                }).await {
                                                                    tracing::error!("Failed to respond after creating db: {e}");
                                                                }
                                                            }
                                                            Err(e) => {
                                                                tracing::error!("Failed to create category: {e}");
                                                                if let Err(e) = command.create_interaction_response(&ctx.http, |resp| {
                                                                    resp.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|m| m.content("Failed to create database. Check bot permissions."))
                                                                }).await {
                                                                    tracing::error!("Failed to send error response: {e}");
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        if let Err(e) = command.create_interaction_response(&ctx.http, |resp| {
                                                            resp.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|m| m.content("This command must be used in a server (guild)."))
                                                        }).await {
                                                            tracing::error!("Failed to send guild-only response: {e}");
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            "use" => {
                                // /sql use <name>
                                if let Some(name_opt) = opt.options.get(0) {
                                    if let Some(val) = &name_opt.value {
                                        if let Some(db_name) = val.as_str() {
                                            if let Some(guild_id) = command.guild_id {
                                                // check if category db_<name> exists
                                                match guild_id.channels(&ctx.http).await {
                                                    Ok(chans) => {
                                                        let target = format!("db_{}", db_name);
                                                        let found = chans.values().find(|c| c.name == target && c.kind == ChannelType::Category);
                                                        if let Some(_) = found {
                                                            // store selection in shared data keyed by (guild, user)
                                                            let user_id = command.user.id;
                                                            let data_read = ctx.data.read().await;
                                                            if let Some(map_arc) = data_read.get::<CurrentDB>().cloned() {
                                                                drop(data_read);
                                                                let mut map = map_arc.lock().await;
                                                                map.insert((guild_id, user_id), db_name.to_string());

                                                                if let Err(e) = command.create_interaction_response(&ctx.http, |resp| {
                                                                    resp.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|m| m.content(format!("Using database `db_{}`", db_name)))
                                                                }).await {
                                                                    tracing::error!("Failed to respond after setting current db: {e}");
                                                                }
                                                            } else {
                                                                if let Err(e) = command.create_interaction_response(&ctx.http, |resp| {
                                                                    resp.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|m| m.content("Internal error: data map missing"))
                                                                }).await {
                                                                    tracing::error!("Failed to send internal error response: {e}");
                                                                }
                                                            }
                                                        } else {
                                                            if let Err(e) = command.create_interaction_response(&ctx.http, |resp| {
                                                                resp.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|m| m.content(format!("Database `db_{}` not found", db_name)))
                                                            }).await {
                                                                tracing::error!("Failed to respond for missing db: {e}");
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        tracing::error!("Failed to fetch channels: {e}");
                                                        if let Err(e2) = command.create_interaction_response(&ctx.http, |resp| {
                                                            resp.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|m| m.content("Failed to check databases. Check permissions."))
                                                        }).await {
                                                            tracing::error!("Failed to send channels error response: {e2}");
                                                        }
                                                    }
                                                }
                                            } else {
                                                if let Err(e) = command.create_interaction_response(&ctx.http, |resp| {
                                                    resp.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|m| m.content("This command must be used in a server (guild)."))
                                                }).await {
                                                    tracing::error!("Failed to send guild-only response: {e}");
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {
                    if let Err(e) = command.create_interaction_response(&ctx.http, |resp| {
                        resp.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|m| m.content("Unknown command"))
                    }).await {
                        tracing::error!("Failed to respond to unknown command: {e}");
                    }
                }
            }
        }
    }
}
