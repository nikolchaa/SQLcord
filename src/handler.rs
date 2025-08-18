use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::application::Interaction;
use serenity::model::application::CommandDataOptionValue;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
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
                                                                        match crate::commands::sql::create::db::run(&ctx, guild_id, db_name).await {
                                                                            Ok(msg) => {
                                                                                if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                                    CreateInteractionResponseMessage::new().content(msg)
                                                                                )).await {
                                                                                    tracing::error!("Failed to respond after creating db: {e}");
                                                                                }
                                                                            }
                                                                            Err(err_msg) => {
                                                                                if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                                    CreateInteractionResponseMessage::new().content(err_msg)
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
                                            } else if sub.name == "table" {
                                                if let CommandDataOptionValue::SubCommand(params) = &sub.value {
                                                    if let Some(name_opt) = params.get(0) {
                                                        if let CommandDataOptionValue::String(table_name) = &name_opt.value {
                                                            if let Some(guild_id) = command.guild_id {
                                                                let user_id = command.user.id;
                                                                match crate::commands::sql::create::table::run(&ctx, guild_id, user_id, table_name).await {
                                                                    Ok(msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(msg)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after creating table: {e}");
                                                                        }
                                                                    }
                                                                    Err(err_msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(err_msg)
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
                                                                match crate::commands::sql::create::db::run(&ctx, guild_id, db_name).await {
                                                                    Ok(msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(msg)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after creating db: {e}");
                                                                        }
                                                                    }
                                                                    Err(err_msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(err_msg)
                                                                        )).await {
                                                                            tracing::error!("Failed to send error response: {e}");
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            } else if sub.name == "table" {
                                                if let CommandDataOptionValue::SubCommand(inner) = &sub.value {
                                                    if let Some(name_opt) = inner.get(0) {
                                                        if let CommandDataOptionValue::String(table_name) = &name_opt.value {
                                                            if let Some(guild_id) = command.guild_id {
                                                                let user_id = command.user.id;
                                                                match crate::commands::sql::create::table::run(&ctx, guild_id, user_id, table_name).await {
                                                                    Ok(msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(msg)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after creating table: {e}");
                                                                        }
                                                                    }
                                                                    Err(err_msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(err_msg)
                                                                        )).await {
                                                                            tracing::error!("Failed to send error response: {e}");
                                                                        }
                                                                    }
                                                                }
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
                            "drop" => {
                                match &opt.value {
                                    CommandDataOptionValue::SubCommandGroup(groups) => {
                                        if let Some(sub) = groups.get(0) {
                                            if sub.name == "db" {
                                                if let CommandDataOptionValue::SubCommand(params) = &sub.value {
                                                    if let Some(name_opt) = params.get(0) {
                                                        if let CommandDataOptionValue::String(db_name) = &name_opt.value {
                                                            if let Some(guild_id) = command.guild_id {
                                                                match crate::commands::sql::drop::db::run(&ctx, guild_id, db_name).await {
                                                                    Ok(msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(msg)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after dropping db: {e}");
                                                                        }
                                                                    }
                                                                    Err(err_msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(err_msg)
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
                                            } else if sub.name == "table" {
                                                if let CommandDataOptionValue::SubCommand(params) = &sub.value {
                                                    if let Some(name_opt) = params.get(0) {
                                                        if let CommandDataOptionValue::String(table_name) = &name_opt.value {
                                                            if let Some(guild_id) = command.guild_id {
                                                                let user_id = command.user.id;
                                                                match crate::commands::sql::drop::table::run(&ctx, guild_id, user_id, table_name).await {
                                                                    Ok(msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(msg)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after dropping table: {e}");
                                                                        }
                                                                    }
                                                                    Err(err_msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(err_msg)
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
                                        // handle if drop was registered as subcommand directly
                                        if let Some(sub) = params.get(0) {
                                            if sub.name == "db" {
                                                if let CommandDataOptionValue::SubCommand(inner) = &sub.value {
                                                    if let Some(name_opt) = inner.get(0) {
                                                        if let CommandDataOptionValue::String(db_name) = &name_opt.value {
                                                            if let Some(guild_id) = command.guild_id {
                                                                match crate::commands::sql::drop::db::run(&ctx, guild_id, db_name).await {
                                                                    Ok(msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(msg)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after dropping db: {e}");
                                                                        }
                                                                    }
                                                                    Err(err_msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(err_msg)
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
                                            } else if sub.name == "table" {
                                                if let CommandDataOptionValue::SubCommand(inner) = &sub.value {
                                                    if let Some(name_opt) = inner.get(0) {
                                                        if let CommandDataOptionValue::String(table_name) = &name_opt.value {
                                                            if let Some(guild_id) = command.guild_id {
                                                                let user_id = command.user.id;
                                                                match crate::commands::sql::drop::table::run(&ctx, guild_id, user_id, table_name).await {
                                                                    Ok(msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(msg)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after dropping table: {e}");
                                                                        }
                                                                    }
                                                                    Err(err_msg) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().content(err_msg)
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
                                    _ => {}
                                }
                            }
                            "use" => {
                                match &opt.value {
                                    CommandDataOptionValue::SubCommand(params) => {
                                        if let Some(name_opt) = params.get(0) {
                                            if let CommandDataOptionValue::String(db_name) = &name_opt.value {
                                                if let Some(guild_id) = command.guild_id {
                                                    let user_id = command.user.id;
                                                    match crate::commands::sql::use_::run(&ctx, guild_id, user_id, db_name).await {
                                                        Ok(msg) => {
                                                            if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                CreateInteractionResponseMessage::new().content(msg)
                                                            )).await {
                                                                tracing::error!("Failed to respond after setting current db: {e}");
                                                            }
                                                        }
                                                        Err(err_msg) => {
                                                            if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                CreateInteractionResponseMessage::new().content(err_msg)
                                                            )).await {
                                                                tracing::error!("Failed to send internal error response: {e}");
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
