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
                                                                            Ok(embed) => {
                                                                                if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                                    CreateInteractionResponseMessage::new().embed(embed)
                                                                                )).await {
                                                                                    tracing::error!("Failed to respond after creating db: {e}");
                                                                                }
                                                                            }
                                                                            Err(embed) => {
                                                                                if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                                    CreateInteractionResponseMessage::new().embed(embed)
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
                                                            // Extract optional schema parameter
                                                            let schema = params.get(1).and_then(|opt| {
                                                                if let CommandDataOptionValue::String(schema_str) = &opt.value {
                                                                    Some(schema_str.as_str())
                                                                } else {
                                                                    None
                                                                }
                                                            });
                                                            
                                                            if let Some(guild_id) = command.guild_id {
                                                                let user_id = command.user.id;
                                                                match crate::commands::sql::create::table::run(&ctx, guild_id, user_id, table_name, schema).await {
                                                                    Ok(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after creating table: {e}");
                                                                        }
                                                                    }
                                                                    Err(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
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
                                                                    Ok(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after creating db: {e}");
                                                                        }
                                                                    }
                                                                    Err(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
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
                                                            // Extract optional schema parameter
                                                            let schema = inner.get(1).and_then(|opt| {
                                                                if let CommandDataOptionValue::String(schema_str) = &opt.value {
                                                                    Some(schema_str.as_str())
                                                                } else {
                                                                    None
                                                                }
                                                            });
                                                            
                                                            if let Some(guild_id) = command.guild_id {
                                                                let user_id = command.user.id;
                                                                match crate::commands::sql::create::table::run(&ctx, guild_id, user_id, table_name, schema).await {
                                                                    Ok(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after creating table: {e}");
                                                                        }
                                                                    }
                                                                    Err(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
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
                                                                    Ok(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after dropping db: {e}");
                                                                        }
                                                                    }
                                                                    Err(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
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
                                                                    Ok(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after dropping table: {e}");
                                                                        }
                                                                    }
                                                                    Err(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
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
                                                                    Ok(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after dropping db: {e}");
                                                                        }
                                                                    }
                                                                    Err(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
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
                                                                    Ok(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
                                                                        )).await {
                                                                            tracing::error!("Failed to respond after dropping table: {e}");
                                                                        }
                                                                    }
                                                                    Err(embed) => {
                                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new().embed(embed)
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
                                                        Ok(embed) => {
                                                            if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                CreateInteractionResponseMessage::new().embed(embed)
                                                            )).await {
                                                                tracing::error!("Failed to respond after setting current db: {e}");
                                                            }
                                                        }
                                                        Err(embed) => {
                                                            if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                CreateInteractionResponseMessage::new().embed(embed)
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
                            "explain" => {
                                match &opt.value {
                                    CommandDataOptionValue::SubCommand(params) => {
                                        if let Some(operation_opt) = params.get(0) {
                                            if let CommandDataOptionValue::String(operation) = &operation_opt.value {
                                                match crate::commands::sql::explain::run(operation).await {
                                                    Ok(embed) => {
                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                            CreateInteractionResponseMessage::new().embed(embed)
                                                        )).await {
                                                            tracing::error!("Failed to respond with explanation: {e}");
                                                        }
                                                    }
                                                    Err(embed) => {
                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                            CreateInteractionResponseMessage::new().embed(embed)
                                                        )).await {
                                                            tracing::error!("Failed to send explain error response: {e}");
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            "select" => {
                                match &opt.value {
                                    CommandDataOptionValue::SubCommand(params) => {
                                        // Extract parameters
                                        let mut columns = None;
                                        let mut table = None;
                                        let mut distinct = None;
                                        let mut where_clause = None;
                                        
                                        for param in params {
                                            match param.name.as_str() {
                                                "columns" => {
                                                    if let CommandDataOptionValue::String(cols) = &param.value {
                                                        columns = Some(cols.as_str());
                                                    }
                                                }
                                                "from" => {
                                                    if let CommandDataOptionValue::String(tbl) = &param.value {
                                                        table = Some(tbl.as_str());
                                                    }
                                                }
                                                "distinct" => {
                                                    if let CommandDataOptionValue::Boolean(dist) = &param.value {
                                                        distinct = Some(*dist);
                                                    }
                                                }
                                                "where" => {
                                                    if let CommandDataOptionValue::String(whr) = &param.value {
                                                        where_clause = Some(whr.as_str());
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        
                                        if let (Some(columns), Some(table)) = (columns, table) {
                                            if let Some(guild_id) = command.guild_id {
                                                let user_id = command.user.id;
                                                match crate::commands::sql::select::run(&ctx, guild_id, user_id, columns, table, distinct, where_clause).await {
                                                    Ok(embed) => {
                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                            CreateInteractionResponseMessage::new().embed(embed)
                                                        )).await {
                                                            tracing::error!("Failed to respond after selecting data: {e}");
                                                        }
                                                    }
                                                    Err(embed) => {
                                                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                            CreateInteractionResponseMessage::new().embed(embed)
                                                        )).await {
                                                            tracing::error!("Failed to send select error response: {e}");
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
                                        } else {
                                            if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new().content("Missing required parameters: columns and table name.")
                                            )).await {
                                                tracing::error!("Failed to send parameter error response: {e}");
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            "insert" => {
                                match &opt.value {
                                    CommandDataOptionValue::SubCommandGroup(groups) => {
                                        if let Some(sub) = groups.get(0) {
                                            if sub.name == "into" {
                                                if let CommandDataOptionValue::SubCommand(params) = &sub.value {
                                                    if let Some(table_opt) = params.get(0) {
                                                        if let CommandDataOptionValue::String(table_name) = &table_opt.value {
                                                            if let Some(data_opt) = params.get(1) {
                                                                if let CommandDataOptionValue::String(data) = &data_opt.value {
                                                                    if let Some(guild_id) = command.guild_id {
                                                                        let user_id = command.user.id;
                                                                        match crate::commands::sql::insert::run(&ctx, guild_id, user_id, table_name, data).await {
                                                                            Ok(embed) => {
                                                                                if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                                    CreateInteractionResponseMessage::new().embed(embed)
                                                                                )).await {
                                                                                    tracing::error!("Failed to respond after inserting data: {e}");
                                                                                }
                                                                            }
                                                                            Err(embed) => {
                                                                                if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                                                                    CreateInteractionResponseMessage::new().embed(embed)
                                                                                )).await {
                                                                                    tracing::error!("Failed to send insert error response: {e}");
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
