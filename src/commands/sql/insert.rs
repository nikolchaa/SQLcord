// /sql insert into <table> <data>

use std::error::Error;
use serenity::prelude::Context;
use serenity::model::id::{GuildId, UserId};
use serenity::model::channel::ChannelType;
use serenity::builder::CreateMessage;
use crate::state::CurrentDB;
use crate::logging::{log_info, log_error};
use crate::utils::{sanitize_channel_name, create_success_embed, create_error_embed};
use crate::sql_parser::{parse_column_definitions, ColumnDefinition, parse_sql_values, validate_values_against_schema, SqlValue};

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering INSERT command");
    Ok(())
}

/// Insert data into a table (Discord channel)
/// Validates data against table schema and stores as a message
pub async fn run(ctx: &Context, guild_id: GuildId, user_id: UserId, table_name: &str, data: &str) -> Result<serenity::builder::CreateEmbed, serenity::builder::CreateEmbed> {
    log_info(&format!("INSERT command executed for table: {} with data: {}", table_name, data));
    
    // Parse and validate SQL VALUES data
    let parsed_values = match parse_sql_values(data) {
        Ok(values) => values,
        Err(e) => {
            let embed = create_error_embed(
                "✖️ Invalid Data Format",
                &format!("**Data Error:**\n{}\n\n💡 **Tip:** Use SQL format like `1, 'John', true`", e)
            );
            return Err(embed);
        }
    };
    
    // Sanitize the table name
    let (sanitized_name, _) = sanitize_channel_name(table_name);
    
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

    // Find the table channel and validate data against schema
    match guild_id.channels(&ctx.http).await {
        Ok(channels) => {
            let table_channel_name = format!("table_{}", sanitized_name);
            let db_category_name = format!("db_{}", current_db);
            
            // Find the database category
            let db_category = channels.values()
                .find(|c| c.name == db_category_name && c.kind == ChannelType::Category);
            
            if let Some(category) = db_category {
                // Find the table channel
                let table_channel = channels.values()
                    .find(|c| c.name == table_channel_name && c.parent_id == Some(category.id));
                
                if let Some(channel) = table_channel {
                    // Get and parse table schema from channel topic
                    let schema = if let Some(topic) = &channel.topic {
                        parse_schema_from_topic(topic)?
                    } else {
                        Vec::new() // No schema defined
                    };
                    
                    // Validate data against schema
                    if let Err(validation_error) = validate_values_against_schema(&parsed_values, &schema) {
                        return Err(create_error_embed(
                            "✖️ Data Validation Failed",
                            &format!("**Validation Error:**\n{}\n\n💡 **Schema:** {}", validation_error, format_schema_info(&schema))
                        ));
                    }
                    
                    // Format data for storage
                    let formatted_data = format_sql_values_for_storage(&parsed_values, &schema);
                    
                    // Insert data as a message in the table channel
                    match channel.send_message(&ctx.http, CreateMessage::new().content(&formatted_data)).await {
                        Ok(_message) => {
                            let success_msg = format!(
                                "Successfully inserted 1 row into table **{}**\n\n**Data:**\n{}",
                                sanitized_name,
                                format_sql_values_for_display(&parsed_values, &schema)
                            );
                            log_info(&format!("SUCCESS: Data inserted into table {}", table_channel_name));
                            Ok(create_success_embed("✅ Row Inserted", &success_msg))
                        },
                        Err(e) => {
                            tracing::error!("Failed to insert data into table channel: {e}");
                            let embed = create_error_embed(
                                "✖️ Insert Failed",
                                "Failed to insert data. Please check bot permissions or try again."
                            );
                            log_error("Failed to insert data");
                            Err(embed)
                        }
                    }
                } else {
                    let embed = create_error_embed(
                        "✖️ Table Not Found",
                        &format!("Table **{}** not found in database **{}**. Create it first with `/sql create table {}`", sanitized_name, current_db, sanitized_name)
                    );
                    Err(embed)
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

/// Format SQL values for storage in Discord message
fn format_sql_values_for_storage(values: &[SqlValue], schema: &[ColumnDefinition]) -> String {
    let mut parts = Vec::new();
    
    // Add timestamp
    parts.push(format!("TIMESTAMP: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Add data in a structured format
    parts.push("DATA:".to_string());
    
    if schema.is_empty() {
        // No schema - just format values by position
        for (i, value) in values.iter().enumerate() {
            parts.push(format!("  column_{}: {}", i + 1, format_sql_value_for_display(value)));
        }
    } else {
        // Format according to schema order
        for (column, value) in schema.iter().zip(values.iter()) {
            parts.push(format!("  {}: {}", column.name, format_sql_value_for_display(value)));
        }
        
        // Add any extra values beyond schema
        if values.len() > schema.len() {
            for (i, value) in values.iter().skip(schema.len()).enumerate() {
                parts.push(format!("  extra_{}: {}", i + 1, format_sql_value_for_display(value)));
            }
        }
    }
    
    parts.join("\n")
}

/// Format SQL values for user-friendly display
fn format_sql_values_for_display(values: &[SqlValue], schema: &[ColumnDefinition]) -> String {
    if schema.is_empty() {
        // No schema - just format values by position
        values.iter()
            .enumerate()
            .map(|(i, value)| format!("• **Column {}:** {}", i + 1, format_sql_value_for_display(value)))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        // Use schema column names
        schema.iter()
            .zip(values.iter())
            .map(|(column, value)| format!("• **{}:** {}", column.name, format_sql_value_for_display(value)))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Format a single SQL value for display
fn format_sql_value_for_display(value: &SqlValue) -> String {
    match value {
        SqlValue::String(s) => format!("\"{}\"", s),
        SqlValue::Integer(n) => n.to_string(),
        SqlValue::Float(f) => f.to_string(),
        SqlValue::Boolean(b) => b.to_string(),
        SqlValue::Null => "NULL".to_string(),
    }
}

/// Parse table schema from channel topic
fn parse_schema_from_topic(topic: &str) -> Result<Vec<ColumnDefinition>, serenity::builder::CreateEmbed> {
    if let Some(schema_start) = topic.find("Schema: ") {
        let schema_str = &topic[schema_start + 8..];
        
        // Handle backward compatibility: if the schema contains colons (old format),
        // convert it to the new format before parsing
        let normalized_schema = if schema_str.contains(": ") {
            // Old format: "id: INT, name: VARCHAR" -> "id INT, name VARCHAR"
            schema_str.replace(": ", " ")
        } else {
            // New format: already correct
            schema_str.to_string()
        };
        
        match parse_column_definitions(&normalized_schema) {
            Ok(columns) => Ok(columns),
            Err(e) => {
                Err(create_error_embed(
                    "✖️ Schema Parse Error",
                    &format!("Failed to parse table schema: {}", e)
                ))
            }
        }
    } else {
        Ok(Vec::new()) // No schema in topic
    }
}

/// Format schema information for display
fn format_schema_info(schema: &[ColumnDefinition]) -> String {
    if schema.is_empty() {
        "No schema defined (flexible insertion allowed)".to_string()
    } else {
        let column_info: Vec<String> = schema.iter()
            .map(|col| {
                let mut info = format!("{} {}", col.name, col.data_type);
                if let Some(size) = col.size {
                    info += &format!("({})", size);
                }
                if !col.nullable {
                    info += " NOT NULL";
                }
                if col.primary_key {
                    info += " PRIMARY KEY";
                }
                info
            })
            .collect();
        column_info.join(", ")
    }
}

// Essential functionality only - no tests needed
