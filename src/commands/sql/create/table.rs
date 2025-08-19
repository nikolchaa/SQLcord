// /sql create table <name> [schema]

use std::error::Error;
use serenity::prelude::Context;
use serenity::model::id::{GuildId, UserId};
use serenity::model::channel::ChannelType;
use crate::state::CurrentDB;
use crate::logging::{log_info, log_error};
use crate::utils::{sanitize_channel_name, create_success_embed, create_error_embed};
use crate::sql_parser::parse_column_definitions;

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering CREATE TABLE command");
    Ok(())
}

/// Create a text channel named `table_<table_name>` under the current database category.
/// If schema is provided, parse and store the column definitions.
/// Returns Ok(embed) or Err(embed).
pub async fn run(ctx: &Context, guild_id: GuildId, user_id: UserId, table_name: &str, schema: Option<&str>) -> Result<serenity::builder::CreateEmbed, serenity::builder::CreateEmbed> {
    log_info(&format!("CREATE TABLE command executed for table: {} with schema: {:?}", table_name, schema));
    
    // Parse schema if provided
    let parsed_schema = if let Some(schema_str) = schema {
        match parse_column_definitions(schema_str) {
            Ok(columns) => Some(columns),
            Err(e) => {
                let embed = create_error_embed(
                    "✖️ Invalid Table Schema",
                    &format!("**Schema Error:**\n{}\n\n💡 **Tip:** Use formats like `id INT`, `name VARCHAR(255)`, `active BOOLEAN`", e)
                );
                return Err(embed);
            }
        }
    } else {
        None
    };
    
    // Sanitize the table name
    let (sanitized_name, was_changed) = sanitize_channel_name(table_name);
    
    if sanitized_name.is_empty() {
        let embed = create_error_embed(
            "✖️ Invalid Table Name",
            "Table name cannot be empty after sanitization. Please provide a valid name with alphanumeric characters."
        );
        return Err(embed);
    }
    
    // Get the current database for this user
    let data_read = ctx.data.read().await;
    let current_db = if let Some(map_arc) = data_read.get::<CurrentDB>() {
        let map = map_arc.lock().await;
        map.get(&(guild_id, user_id)).cloned()
    } else {
        None
    };
    drop(data_read);

    let current_db = match current_db {
        Some(db_name) => db_name,
        None => {
            let embed = create_error_embed(
                "✖️ No Database Selected",
                "No database selected. Use `/sql use <db_name>` first to select a database."
            );
            return Err(embed);
        }
    };

    // Find the database category
    match guild_id.channels(&ctx.http).await {
        Ok(channels) => {
            let db_category_name = format!("db_{}", current_db);
            let db_category = channels.values()
                .find(|c| c.name == db_category_name && c.kind == ChannelType::Category);
            
            if let Some(category) = db_category {
                // Check if table already exists
                let table_channel_name = format!("table_{}", sanitized_name);
                let existing_table = channels.values()
                    .find(|c| c.name == table_channel_name && c.parent_id == Some(category.id));
                
                if existing_table.is_some() {
                    let embed = create_error_embed(
                        "✖️ Table Already Exists",
                        &format!("Table **{}** already exists in database **{}**", sanitized_name, current_db)
                    );
                    return Err(embed);
                }

                // Create the table channel
                let mut builder = serenity::builder::CreateChannel::new(&table_channel_name)
                    .kind(ChannelType::Text)
                    .category(category.id);
                
                // Add schema to channel topic if provided
                if let Some(columns) = &parsed_schema {
                    let schema_description = columns.iter()
                        .map(|col| format!("{} {}", col.name, col.data_type))
                        .collect::<Vec<_>>()
                        .join(", ");
                    builder = builder.topic(&format!("Schema: {}", schema_description));
                }
                
                match guild_id.create_channel(&ctx.http, builder).await {
                    Ok(_channel) => {
                        let mut description = format!("Table **{}** created in database **{}**", sanitized_name, current_db);
                        if was_changed {
                            description.push_str(&format!("\n\n*Name sanitized from `{}` to `{}`*", table_name, sanitized_name));
                        }
                        
                        // Add schema information to success message
                        if let Some(columns) = &parsed_schema {
                            description.push_str("\n\n**Schema:**\n");
                            for column in columns {
                                description.push_str(&format!("• {}\n", column));
                            }
                        }
                        
                        let embed = create_success_embed("✔️ Table Created", &description);
                        log_info(&format!("SUCCESS: Table {} created with {} columns", table_channel_name, parsed_schema.as_ref().map_or(0, |s| s.len())));
                        Ok(embed)
                    },
                    Err(e) => {
                        tracing::error!("Failed to create table channel: {e}");
                        let embed = create_error_embed(
                            "✖️ Table Creation Failed",
                            "Failed to create table. Please check bot permissions or try again."
                        );
                        log_error("Failed to create table");
                        Err(embed)
                    }
                }
            } else {
                let embed = create_error_embed(
                    "✖️ Database Not Found",
                    &format!("Database **{}** not found. Create it first with `/sql create db {}`", current_db, current_db)
                );
                Err(embed)
            }
        },
        Err(e) => {
            tracing::error!("Failed to get channels: {e}");
            let embed = create_error_embed(
                "✖️ Permission Error",
                "Failed to list channels. Please check bot permissions."
            );
            Err(embed)
        }
    }
}
