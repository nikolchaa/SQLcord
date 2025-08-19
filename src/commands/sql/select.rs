// /sql select <columns> from <table> [distinct] [where]

use std::error::Error;
use std::collections::{HashMap, HashSet};
use serenity::prelude::Context;
use serenity::model::id::{GuildId, UserId};
use serenity::model::channel::ChannelType;
use crate::state::CurrentDB;
use crate::logging::log_info;
use crate::utils::{sanitize_channel_name, create_error_embed, create_info_embed};
use crate::sql_parser::{parse_column_definitions, ColumnDefinition, SqlValue};

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering SELECT command");
    Ok(())
}

/// SELECT data from a table (Discord channel)
/// Supports column selection, DISTINCT, and enhanced WHERE filtering
pub async fn run(
    ctx: &Context, 
    guild_id: GuildId, 
    user_id: UserId, 
    columns: &str, 
    table_name: &str,
    distinct: Option<bool>,
    where_clause: Option<&str>
) -> Result<serenity::builder::CreateEmbed, serenity::builder::CreateEmbed> {
    log_info(&format!("SELECT command executed: columns={}, table={}, distinct={:?}, where={:?}", 
                      columns, table_name, distinct, where_clause));
    
    // Get the current database for this user
    let current_db_key = (guild_id, user_id);
    let current_db = {
        let data = ctx.data.read().await;
        if let Some(db_store) = data.get::<CurrentDB>() {
            let db_map = db_store.lock().await;
            db_map.get(&current_db_key).cloned()
        } else {
            None
        }
    };
    
    let current_db = match current_db {
        Some(db) => db,
        None => {
            return Err(create_error_embed(
                "✖️ No Database Selected",
                "Please select a database first using `/sql use <database_name>`"
            ));
        }
    };
    
    // Get categories in the guild
    let channels = match guild_id.channels(&ctx.http).await {
        Ok(channels) => channels,
        Err(_) => {
            return Err(create_error_embed(
                "✖️ Database Access Error",
                "Could not access guild channels. Please check bot permissions."
            ));
        }
    };
    
    let categories = channels
        .values()
        .filter(|c| c.kind == ChannelType::Category)
        .collect::<Vec<_>>();
    
    // Find the current database category
    let db_category_name = format!("db_{}", current_db);
    let category = categories
        .iter()
        .find(|c| c.name == db_category_name)
        .ok_or_else(|| {
            create_error_embed(
                "✖️ Database Not Found",
                &format!("Database **{}** does not exist. Please create it first or select a different database.", current_db)
            )
        })?;
    
    // Find the table channel within the category
    let (sanitized_table_name, _) = sanitize_channel_name(table_name);
    let table_channel_name = format!("table_{}", sanitized_table_name);
    
    let all_channels = match guild_id.channels(&ctx.http).await {
        Ok(channels) => channels,
        Err(_) => {
            return Err(create_error_embed(
                "✖️ Channel Access Error",
                "Could not access guild channels. Please check bot permissions."
            ));
        }
    };
    
    let table_channel = all_channels
        .values()
        .find(|c| c.name == table_channel_name && c.parent_id == Some(category.id))
        .ok_or_else(|| {
            create_error_embed(
                "✖️ Table Not Found",
                &format!("Table **{}** does not exist in database **{}**. Please create it first.", table_name, current_db)
            )
        })?;
    
    // Get and parse table schema from channel topic
    let schema = if let Some(topic) = &table_channel.topic {
        parse_schema_from_topic(topic)?
    } else {
        Vec::new() // No schema defined
    };
    
    // Parse column selection
    let selected_columns = parse_column_selection(columns, &schema)?;
    
    // Fetch messages from the table channel
    let messages = match table_channel.messages(&ctx.http, serenity::builder::GetMessages::new().limit(100)).await {
        Ok(messages) => messages,
        Err(_) => {
            return Err(create_error_embed(
                "✖️ Table Access Error",
                "Could not read messages from table. Please check bot permissions."
            ));
        }
    };
    
    // Extract and filter data
    let mut rows = Vec::new();
    for message in messages.iter().rev() { // Reverse to show oldest first
        if let Some(row_data) = extract_values_from_message(&message.content, &schema) {
            // Apply WHERE filtering if specified
            if let Some(where_condition) = where_clause {
                if !evaluate_where_condition(&row_data, &schema, where_condition) {
                    continue;
                }
            }
            
            // Select only requested columns
            let selected_row = select_columns(&row_data, &schema, &selected_columns);
            rows.push(selected_row);
        }
    }
    
    // Apply DISTINCT if requested
    if distinct.unwrap_or(false) {
        rows = apply_distinct(rows);
    }
    
    // Format results
    let result_embed = format_select_results(&selected_columns, &rows, table_name, distinct.unwrap_or(false), where_clause);
    Ok(result_embed)
}

/// Parse schema from channel topic (similar to insert.rs)
fn parse_schema_from_topic(topic: &str) -> Result<Vec<ColumnDefinition>, serenity::builder::CreateEmbed> {
    if let Some(schema_start) = topic.find("Schema: ") {
        let schema_str = &topic[schema_start + 8..];
        
        // Handle backward compatibility: if the schema contains colons (old format),
        // convert it to the new format before parsing
        let normalized_schema = if schema_str.contains(": ") {
            schema_str.replace(": ", " ")
        } else {
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

/// Parse column selection (*, column names, etc.)
fn parse_column_selection(columns: &str, schema: &[ColumnDefinition]) -> Result<Vec<String>, serenity::builder::CreateEmbed> {
    let columns = columns.trim();
    
    if columns == "*" {
        // Select all columns
        if schema.is_empty() {
            return Err(create_error_embed(
                "✖️ Schema Required",
                "Cannot use '*' selection on tables without defined schema. Please specify column names explicitly."
            ));
        }
        Ok(schema.iter().map(|col| col.name.clone()).collect())
    } else {
        // Parse specific column names
        let requested_columns: Vec<String> = columns
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if requested_columns.is_empty() {
            return Err(create_error_embed(
                "✖️ Invalid Column Selection",
                "Please specify column names or use '*' to select all columns."
            ));
        }
        
        // Validate column names against schema (if schema exists)
        if !schema.is_empty() {
            let schema_columns: HashSet<String> = schema.iter().map(|col| col.name.clone()).collect();
            for col in &requested_columns {
                if !schema_columns.contains(col) {
                    return Err(create_error_embed(
                        "✖️ Unknown Column",
                        &format!("Column **{}** does not exist in table schema.\n\n**Available columns:** {}", 
                                col, schema.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", "))
                    ));
                }
            }
        }
        
        Ok(requested_columns)
    }
}

/// Extract values from stored message (similar to insert.rs)
fn extract_values_from_message(content: &str, schema: &[ColumnDefinition]) -> Option<Vec<SqlValue>> {
    if let Some(data_start) = content.find("DATA:\n") {
        let data_section = &content[data_start + 6..];
        let mut value_map = HashMap::new();
        
        for line in data_section.lines() {
            if line.starts_with("  ") && line.contains(": ") {
                if let Some(colon_pos) = line.find(": ") {
                    let column_name = line[2..colon_pos].trim();
                    let value_str = line[colon_pos + 2..].trim();
                    
                    if let Ok(sql_value) = parse_stored_value(value_str) {
                        value_map.insert(column_name.to_string(), sql_value);
                    }
                }
            }
        }
        
        // If we have a schema, use it to order values
        if !schema.is_empty() {
            let mut ordered_values = Vec::new();
            for column in schema {
                if let Some(value) = value_map.get(&column.name) {
                    ordered_values.push(value.clone());
                } else {
                    return None; // Missing column
                }
            }
            if ordered_values.len() == schema.len() {
                return Some(ordered_values);
            }
        } else {
            // No schema - return values in order found
            return Some(value_map.into_values().collect());
        }
    }
    None
}

/// Parse stored value back to SqlValue (similar to insert.rs)
fn parse_stored_value(value_str: &str) -> Result<SqlValue, String> {
    let trimmed = value_str.trim();
    
    if trimmed.eq_ignore_ascii_case("null") {
        return Ok(SqlValue::Null);
    }
    
    if trimmed.eq_ignore_ascii_case("true") {
        return Ok(SqlValue::Boolean(true));
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return Ok(SqlValue::Boolean(false));
    }
    
    if (trimmed.starts_with('\'') && trimmed.ends_with('\'')) || 
       (trimmed.starts_with('"') && trimmed.ends_with('"')) {
        let content = &trimmed[1..trimmed.len()-1];
        return Ok(SqlValue::String(content.to_string()));
    }
    
    if let Ok(int_val) = trimmed.parse::<i64>() {
        return Ok(SqlValue::Integer(int_val));
    }
    
    if let Ok(float_val) = trimmed.parse::<f64>() {
        return Ok(SqlValue::Float(float_val));
    }
    
    // Default to string if nothing else matches
    Ok(SqlValue::String(trimmed.to_string()))
}

/// Select only requested columns from a row
fn select_columns(row_data: &[SqlValue], schema: &[ColumnDefinition], selected_columns: &[String]) -> Vec<SqlValue> {
    if schema.is_empty() {
        // Without schema, just return first N values
        return row_data.iter().take(selected_columns.len()).cloned().collect();
    }
    
    let mut result = Vec::new();
    for col_name in selected_columns {
        if let Some(index) = schema.iter().position(|col| &col.name == col_name) {
            if let Some(value) = row_data.get(index) {
                result.push(value.clone());
            } else {
                result.push(SqlValue::Null);
            }
        }
    }
    result
}

/// Apply DISTINCT filtering
fn apply_distinct(rows: Vec<Vec<SqlValue>>) -> Vec<Vec<SqlValue>> {
    let mut seen = HashSet::new();
    let mut distinct_rows = Vec::new();
    
    for row in rows {
        let row_key = format!("{:?}", row); // Simple serialization for comparison
        if seen.insert(row_key) {
            distinct_rows.push(row);
        }
    }
    
    distinct_rows
}

/// Enhanced WHERE condition evaluation with AND/OR and parentheses support
fn evaluate_where_condition(
    row_data: &[SqlValue], 
    schema: &[ColumnDefinition], 
    where_condition: &str
) -> bool {
    // Support AND/OR logic with parentheses
    // Format examples: 
    // - "column1='value1' AND column2='value2'"
    // - "column1='value1' OR column2='value2'"
    // - "(name='John' OR name='Jane') AND age='25'"
    // - "name='Admin' OR (category='Electronics' AND price='100')"
    
    parse_or_expression(row_data, schema, where_condition.trim())
}

/// Parse OR expression (lowest precedence)
fn parse_or_expression(
    row_data: &[SqlValue], 
    schema: &[ColumnDefinition], 
    expression: &str
) -> bool {
    let or_parts = split_by_operator(expression, " OR ");
    
    for part in or_parts {
        if parse_and_expression(row_data, schema, part.trim()) {
            return true; // Short-circuit: if any OR part is true, whole expression is true
        }
    }
    
    false
}

/// Parse AND expression (higher precedence than OR)
fn parse_and_expression(
    row_data: &[SqlValue], 
    schema: &[ColumnDefinition], 
    expression: &str
) -> bool {
    let and_parts = split_by_operator(expression, " AND ");
    
    for part in and_parts {
        if !parse_primary_expression(row_data, schema, part.trim()) {
            return false; // Short-circuit: if any AND part is false, whole expression is false
        }
    }
    
    true
}

/// Parse primary expression (parentheses or basic condition)
fn parse_primary_expression(
    row_data: &[SqlValue], 
    schema: &[ColumnDefinition], 
    expression: &str
) -> bool {
    let expr = expression.trim();
    
    if expr.starts_with('(') && expr.ends_with(')') {
        // Remove outer parentheses and evaluate inner expression
        let inner = &expr[1..expr.len()-1];
        return parse_or_expression(row_data, schema, inner);
    }
    
    // Basic condition evaluation
    evaluate_single_condition(row_data, schema, expr)
}

/// Split expression by operator while respecting parentheses
fn split_by_operator<'a>(expression: &'a str, operator: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut current_start = 0;
    let mut paren_depth = 0;
    let chars: Vec<char> = expression.chars().collect();
    let op_chars: Vec<char> = operator.chars().collect();
    
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            _ => {
                // Check if we're at an operator and not inside parentheses
                if paren_depth == 0 && i + op_chars.len() <= chars.len() {
                    let potential_op: String = chars[i..i + op_chars.len()].iter().collect();
                    if potential_op == operator {
                        // Found operator at top level, split here
                        let part = &expression[current_start..i];
                        if !part.trim().is_empty() {
                            parts.push(part);
                        }
                        current_start = i + op_chars.len();
                        i += op_chars.len() - 1; // -1 because loop will increment
                    }
                }
            }
        }
        i += 1;
    }
    
    // Add the remaining part
    let remaining = &expression[current_start..];
    if !remaining.trim().is_empty() {
        parts.push(remaining);
    }
    
    // If no splits were made, return the whole expression
    if parts.is_empty() {
        vec![expression]
    } else {
        parts
    }
}

/// Evaluate a single condition (column=value)
fn evaluate_single_condition(
    row_data: &[SqlValue], 
    schema: &[ColumnDefinition], 
    condition: &str
) -> bool {
    if let Some(eq_pos) = condition.find('=') {
        let column_name = condition[..eq_pos].trim();
        let expected_value = condition[eq_pos + 1..].trim();
        
        if let Some(index) = schema.iter().position(|col| col.name == column_name) {
            if let Some(actual_value) = row_data.get(index) {
                return format_sql_value_for_comparison(actual_value) == expected_value;
            }
        }
    }
    
    // If we can't parse the condition, fail it (fail-closed for security)
    false
}

/// Format SQL value for comparison in WHERE clauses
fn format_sql_value_for_comparison(value: &SqlValue) -> String {
    match value {
        SqlValue::String(s) => format!("'{}'", s),
        SqlValue::Integer(i) => i.to_string(),
        SqlValue::Float(f) => f.to_string(),
        SqlValue::Boolean(b) => b.to_string(),
        SqlValue::Null => "null".to_string(),
    }
}

/// Format SELECT results into a Discord embed
fn format_select_results(
    columns: &[String],
    rows: &[Vec<SqlValue>],
    table_name: &str,
    distinct: bool,
    where_clause: Option<&str>
) -> serenity::builder::CreateEmbed {
    let mut description = String::new();
    
    // Add query info
    description.push_str(&format!("**Table:** {}\n", table_name));
    description.push_str(&format!("**Columns:** {}\n", columns.join(", ")));
    if distinct {
        description.push_str("**Modifier:** DISTINCT\n");
    }
    if let Some(where_cond) = where_clause {
        description.push_str(&format!("**Filter:** WHERE {}\n", where_cond));
    }
    description.push_str(&format!("**Rows returned:** {}\n\n", rows.len()));
    
    if rows.is_empty() {
        description.push_str("*No rows found matching the criteria.*");
    } else {
        // Calculate optimal column widths
        let mut col_widths = vec![3; columns.len() + 1]; // Start with minimum widths, +1 for Row column
        col_widths[0] = std::cmp::max(3, "Row".len()); // Row column
        
        // Set minimum width based on column names
        for (i, col) in columns.iter().enumerate() {
            col_widths[i + 1] = std::cmp::max(col_widths[i + 1], col.len());
        }
        
        // Calculate widths based on actual data (limit to first 20 rows for performance)
        let display_rows = rows.iter().take(20).collect::<Vec<_>>();
        for (row_idx, row) in display_rows.iter().enumerate() {
            // Update width for row number column
            let row_num_width = (row_idx + 1).to_string().len();
            col_widths[0] = std::cmp::max(col_widths[0], row_num_width);
            
            // Update widths for data columns
            for (col_idx, value) in row.iter().enumerate() {
                let formatted = format_sql_value_for_display_table(value);
                if col_idx + 1 < col_widths.len() {
                    col_widths[col_idx + 1] = std::cmp::max(col_widths[col_idx + 1], formatted.len());
                }
            }
        }
        
        // Apply maximum width limit to prevent extremely wide tables
        const MAX_COL_WIDTH: usize = 50;
        for width in &mut col_widths {
            *width = std::cmp::min(*width, MAX_COL_WIDTH);
        }
        
        // Build the table
        description.push_str("```\n");
        
        // Header row
        description.push_str(&format!("{:<width$}", "Row", width = col_widths[0]));
        for (i, col) in columns.iter().enumerate() {
            description.push_str(&format!(" | {:<width$}", col, width = col_widths[i + 1]));
        }
        description.push_str("\n");
        
        // Separator line
        let total_width = col_widths.iter().sum::<usize>() + (col_widths.len() - 1) * 3; // 3 chars per separator " | "
        description.push_str(&"-".repeat(total_width));
        description.push_str("\n");
        
        // Data rows
        for (row_idx, row) in display_rows.iter().enumerate() {
            description.push_str(&format!("{:<width$}", row_idx + 1, width = col_widths[0]));
            for (col_idx, value) in row.iter().enumerate() {
                let formatted = format_sql_value_for_display_table(value);
                let truncated = if formatted.len() > col_widths[col_idx + 1] {
                    format!("{}...", &formatted[..col_widths[col_idx + 1].saturating_sub(3)])
                } else {
                    formatted
                };
                description.push_str(&format!(" | {:<width$}", truncated, width = col_widths[col_idx + 1]));
            }
            description.push_str("\n");
        }
        
        if rows.len() > 20 {
            description.push_str(&format!("... and {} more rows\n", rows.len() - 20));
        }
        
        description.push_str("```");
        
        // If any values were truncated, add a note
        let has_long_values = display_rows.iter().any(|row| {
            row.iter().any(|value| {
                let formatted = format_sql_value_for_display_table(value);
                formatted.len() > MAX_COL_WIDTH
            })
        });
        
        if has_long_values {
            description.push_str("\n\n*Note: Some long values have been truncated for display. Use more specific column selection to see full values.*");
        }
    }
    
    create_info_embed("📊 SELECT Results", &description)
}

/// Format SQL value for table display (similar to comparison but optimized for tables)
fn format_sql_value_for_display_table(value: &SqlValue) -> String {
    match value {
        SqlValue::String(s) => format!("'{}'", s),
        SqlValue::Integer(i) => i.to_string(),
        SqlValue::Float(f) => f.to_string(),
        SqlValue::Boolean(b) => b.to_string(),
        SqlValue::Null => "NULL".to_string(),
    }
}
