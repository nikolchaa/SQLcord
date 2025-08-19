// /sql insert into <table> <data>

use std::error::Error;
use serenity::prelude::*;
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
                            &format!("**Validation Error:**\n{}\n\n**Schema:** {}", validation_error, format_schema_info(&schema))
                        ));
                    }
                    
                    // Check for primary key duplicates
                    if let Err(duplicate_error) = check_primary_key_duplicates(ctx, channel, &parsed_values, &schema).await {
                        return Err(duplicate_error);
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
                            Ok(create_success_embed("✔️ Row Inserted", &success_msg))
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
        SqlValue::String(s) => format!("'{}'", s),
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

/// Check for primary key duplicates in existing messages
async fn check_primary_key_duplicates(
    ctx: &Context,
    channel: &serenity::model::channel::GuildChannel,
    new_values: &[SqlValue],
    schema: &[ColumnDefinition],
) -> Result<(), serenity::builder::CreateEmbed> {
    // Find primary key column(s)
    let primary_key_columns: Vec<(usize, &ColumnDefinition)> = schema
        .iter()
        .enumerate()
        .filter(|(_, col)| col.primary_key)
        .collect();
    
    // If no primary key defined, no need to check
    if primary_key_columns.is_empty() {
        return Ok(());
    }
    
    // Get primary key values from new data
    let mut new_pk_values = Vec::new();
    for (index, _column) in &primary_key_columns {
        if let Some(value) = new_values.get(*index) {
            new_pk_values.push(value);
        } else {
            return Err(create_error_embed(
                "✖️ Primary Key Missing",
                "Primary key value is required but not provided in the data."
            ));
        }
    }
    
    // Fetch existing messages from the channel
    let messages = match channel.messages(&ctx.http, serenity::builder::GetMessages::new().limit(100)).await {
        Ok(messages) => messages,
        Err(_) => {
            // If we can't read messages, allow the insert (fail-open for permissions issues)
            return Ok(());
        }
    };
    
    // Check each existing message for primary key conflicts
    for message in messages {
        if let Some(existing_values) = extract_values_from_message(&message.content, schema) {
            // Check if primary key values match
            let mut matches = true;
            for (i, (index, _column)) in primary_key_columns.iter().enumerate() {
                if let (Some(new_val), Some(existing_val)) = (new_pk_values.get(i), existing_values.get(*index)) {
                    if !sql_values_equal(new_val, existing_val) {
                        matches = false;
                        break;
                    }
                }
            }
            
            if matches {
                let pk_column_names: Vec<String> = primary_key_columns
                    .iter()
                    .map(|(_, col)| col.name.clone())
                    .collect();
                
                return Err(create_error_embed(
                    "✖️ Primary Key Violation",
                    &format!(
                        "**Duplicate primary key detected!**\n\nPrimary key column(s): **{}**\nValue(s): **{}**\n\n💡 **Tip:** Primary key values must be unique across all rows.",
                        pk_column_names.join(", "),
                        new_pk_values.iter().map(|v| format_sql_value_for_display(v)).collect::<Vec<_>>().join(", ")
                    )
                ));
            }
        }
    }
    
    Ok(())
}

/// Extract values from a stored message in schema order
fn extract_values_from_message(content: &str, schema: &[ColumnDefinition]) -> Option<Vec<SqlValue>> {
    // Look for "DATA:" section
    if let Some(data_start) = content.find("DATA:\n") {
        let data_section = &content[data_start + 6..];
        let mut value_map = std::collections::HashMap::new();
        
        // Parse all column: value pairs
        for line in data_section.lines() {
            // Check if line is indented (starts with spaces) and contains ": "
            if line.starts_with("  ") && line.contains(": ") {
                if let Some(colon_pos) = line.find(": ") {
                    let column_name = line[2..colon_pos].trim();
                    let value_str = line[colon_pos + 2..].trim();
                    
                    // Parse the value string back to SqlValue
                    if let Ok(sql_value) = parse_stored_value(value_str) {
                        value_map.insert(column_name.to_string(), sql_value);
                    }
                }
            }
        }
        
        // Reconstruct values in schema order
        let mut ordered_values = Vec::new();
        for column in schema {
            if let Some(value) = value_map.get(&column.name) {
                ordered_values.push(value.clone());
            } else {
                // Missing column - can't reconstruct properly
                return None;
            }
        }
        
        if ordered_values.len() == schema.len() {
            return Some(ordered_values);
        }
    }
    None
}

/// Parse a stored value string back to SqlValue
fn parse_stored_value(value_str: &str) -> Result<SqlValue, String> {
    let trimmed = value_str.trim();
    
    // Check for NULL
    if trimmed.eq_ignore_ascii_case("null") {
        return Ok(SqlValue::Null);
    }
    
    // Check for boolean
    if trimmed.eq_ignore_ascii_case("true") {
        return Ok(SqlValue::Boolean(true));
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return Ok(SqlValue::Boolean(false));
    }
    
    // Check for string (single or double quotes)
    if (trimmed.starts_with('\'') && trimmed.ends_with('\'')) || 
       (trimmed.starts_with('"') && trimmed.ends_with('"')) {
        let content = &trimmed[1..trimmed.len()-1];
        return Ok(SqlValue::String(content.to_string()));
    }
    
    // Check for integer
    if let Ok(int_val) = trimmed.parse::<i64>() {
        return Ok(SqlValue::Integer(int_val));
    }
    
    // Check for float
    if let Ok(float_val) = trimmed.parse::<f64>() {
        return Ok(SqlValue::Float(float_val));
    }
    
    Err(format!("Cannot parse stored value: {}", value_str))
}

/// Compare two SQL values for equality
fn sql_values_equal(a: &SqlValue, b: &SqlValue) -> bool {
    match (a, b) {
        (SqlValue::Integer(a), SqlValue::Integer(b)) => a == b,
        (SqlValue::Float(a), SqlValue::Float(b)) => (a - b).abs() < f64::EPSILON,
        (SqlValue::String(a), SqlValue::String(b)) => a == b,
        (SqlValue::Boolean(a), SqlValue::Boolean(b)) => a == b,
        (SqlValue::Null, SqlValue::Null) => true,
        _ => false,
    }
}

// Essential functionality only - no tests needed
