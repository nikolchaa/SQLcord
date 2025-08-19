// SQL column definition parsing utilities

use std::fmt;

#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub size: Option<u32>,
    pub nullable: bool,
    pub primary_key: bool,
}

impl fmt::Display for ColumnDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size_str = if let Some(size) = self.size {
            format!("({})", size)
        } else {
            String::new()
        };
        
        let constraints = {
            let mut parts = Vec::new();
            if !self.nullable {
                parts.push("NOT NULL");
            }
            if self.primary_key {
                parts.push("PRIMARY KEY");
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!(" {}", parts.join(" "))
            }
        };
        
        write!(f, "{} {}{}{}", self.name, self.data_type, size_str, constraints)
    }
}

#[derive(Debug)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

impl fmt::Display for TableSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CREATE TABLE {} (\n", self.name)?;
        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 {
                write!(f, ",\n")?;
            }
            write!(f, "    {}", col)?;
        }
        write!(f, "\n)")
    }
}

#[derive(Debug, Clone)]
pub enum SqlValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl fmt::Display for SqlValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlValue::Integer(i) => write!(f, "{}", i),
            SqlValue::Float(fl) => write!(f, "{}", fl),
            SqlValue::String(s) => write!(f, "'{}'", s),
            SqlValue::Boolean(b) => write!(f, "{}", b),
            SqlValue::Null => write!(f, "NULL"),
        }
    }
}

/// Parse SQL-like column definitions
/// Example: "PersonID int, LastName varchar(255), FirstName varchar(255), Address varchar(255), City varchar(255)"
pub fn parse_column_definitions(schema_str: &str) -> Result<Vec<ColumnDefinition>, String> {
    if schema_str.trim().is_empty() {
        return Ok(Vec::new());
    }
    
    let mut columns = Vec::new();
    
    for column_str in schema_str.split(',') {
        let column_str = column_str.trim();
        if column_str.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = column_str.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(format!("Invalid column definition: '{}'. Expected format: 'column_name data_type'", column_str));
        }
        
        let name = parts[0].to_string();
        let mut data_type = parts[1].to_string();
        let mut size = None;
        let mut nullable = true;
        let mut primary_key = false;
        
        // Parse data type with optional size
        if let Some(start) = data_type.find('(') {
            if let Some(end) = data_type.find(')') {
                if let Ok(parsed_size) = data_type[start + 1..end].parse::<u32>() {
                    size = Some(parsed_size);
                    data_type = data_type[..start].to_string();
                }
            }
        }

        // Normalize and validate data type
        let normalized_type = normalize_data_type(&data_type);
        let valid_types = [
            "INT", "VARCHAR", "CHAR", "BOOLEAN", "FLOAT", "DOUBLE", "DECIMAL", "DATE", "TIME", "DATETIME"
        ];
        if !valid_types.contains(&normalized_type.as_str()) {
            return Err(format!(
                "**{}** is not a valid data type for column **{}**\n\n**Supported Types:**\n• INT, VARCHAR, CHAR, BOOLEAN\n• FLOAT, DOUBLE, DECIMAL\n• DATE, TIME, DATETIME\n\n**Examples:** `id INT`, `name VARCHAR(100)`, `active BOOLEAN`",
                data_type,
                name
            ));
        }

        // Validate size constraints for specific data types
        match normalized_type.as_str() {
            "VARCHAR" | "CHAR" => {
                if size.is_none() {
                    return Err(format!(
                        "**{}** requires a size specification for column **{}**\n\n**Examples:**\n• `{} VARCHAR(255)` - variable-length string up to 255 characters\n• `{} CHAR(10)` - fixed-length string of exactly 10 characters\n\n**Common sizes:** VARCHAR(50), VARCHAR(255), VARCHAR(1000), CHAR(1), CHAR(10)",
                        normalized_type,
                        name,
                        name,
                        name
                    ));
                }
                if let Some(s) = size {
                    if s == 0 {
                        return Err(format!(
                            "**{}** size must be greater than 0 for column **{}**\n\n**Examples:** `{} VARCHAR(1)`, `{} CHAR(10)`",
                            normalized_type,
                            name,
                            name,
                            name
                        ));
                    }
                    if s > 65535 {
                        return Err(format!(
                            "**{}** size {} is too large for column **{}** (maximum: 65535)\n\n**Suggestion:** Use a smaller size like `{} VARCHAR(1000)` or consider if you really need such a large text field",
                            normalized_type,
                            s,
                            name,
                            name
                        ));
                    }
                }
            },
            "BOOLEAN" | "DATE" | "TIME" | "DATETIME" => {
                if size.is_some() {
                    return Err(format!(
                        "**{}** does not support size specification for column **{}**\n\n**Correct usage:** `{} {}`\n**Invalid usage:** `{} {}({})`\n\n**Explanation:** {} values have a fixed internal representation and don't need size limits",
                        normalized_type,
                        name,
                        name,
                        normalized_type,
                        name,
                        normalized_type,
                        size.unwrap(),
                        normalized_type
                    ));
                }
            },
            "INT" => {
                if size.is_some() {
                    return Err(format!(
                        "**INT** does not support size specification for column **{}**\n\n**Correct usage:** `{} INT`\n**Invalid usage:** `{} INT({})`\n\n**Explanation:** INT values are fixed-size 64-bit integers and don't need size limits",
                        name,
                        name,
                        name,
                        size.unwrap()
                    ));
                }
            },
            "FLOAT" | "DOUBLE" | "DECIMAL" => {
                if let Some(s) = size {
                    if s == 0 {
                        return Err(format!(
                            "**{}** precision must be greater than 0 for column **{}**\n\n**Examples:** `{} DECIMAL(10)`, `{} FLOAT(7)`",
                            normalized_type,
                            name,
                            name,
                            name
                        ));
                    }
                    if s > 65 {
                        return Err(format!(
                            "**{}** precision {} is too large for column **{}** (maximum: 65)\n\n**Suggestion:** Use a smaller precision like `{} DECIMAL(18)` for most financial calculations",
                            normalized_type,
                            s,
                            name,
                            name
                        ));
                    }
                }
                // FLOAT/DOUBLE/DECIMAL can optionally have precision specified, but it's not required
            },
            _ => {
                // Unknown type - should not reach here due to validation above
            }
        }

        // Check for constraints in remaining parts
        for part in &parts[2..] {
            let part_upper = part.to_uppercase();
            match part_upper.as_str() {
                "NOT" => {
                    // Look for "NOT NULL"
                    if parts.len() > parts.iter().position(|&p| p == *part).unwrap() + 1 {
                        let next_part = parts[parts.iter().position(|&p| p == *part).unwrap() + 1].to_uppercase();
                        if next_part == "NULL" {
                            nullable = false;
                        }
                    }
                },
                "PRIMARY" => {
                    // Look for "PRIMARY KEY"
                    if parts.len() > parts.iter().position(|&p| p == *part).unwrap() + 1 {
                        let next_part = parts[parts.iter().position(|&p| p == *part).unwrap() + 1].to_uppercase();
                        if next_part == "KEY" {
                            primary_key = true;
                        }
                    }
                },
                _ => {}
            }
        }

        columns.push(ColumnDefinition {
            name,
            data_type: normalized_type,
            size,
            nullable,
            primary_key,
        });
    }
    
    Ok(columns)
}

/// Normalize data type names to common SQL standards
fn normalize_data_type(data_type: &str) -> String {
    match data_type.to_lowercase().as_str() {
        "int" | "integer" => "INT".to_string(),
        "varchar" | "string" | "text" => "VARCHAR".to_string(),
        "char" | "character" => "CHAR".to_string(),
        "bool" | "boolean" => "BOOLEAN".to_string(),
        "float" | "real" => "FLOAT".to_string(),
        "double" => "DOUBLE".to_string(),
        "decimal" | "numeric" => "DECIMAL".to_string(),
        "date" => "DATE".to_string(),
        "time" => "TIME".to_string(),
        "datetime" | "timestamp" => "DATETIME".to_string(),
        _ => data_type.to_uppercase(),
    }
}

/// Parse SQL VALUES format like "1, 'John', true, NULL"
/// Returns a vector of parsed SQL values
pub fn parse_sql_values(values_str: &str) -> Result<Vec<SqlValue>, String> {
    let mut values = Vec::new();
    let mut current_value = String::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut chars = values_str.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if escape_next {
            current_value.push(ch);
            escape_next = false;
            continue;
        }
        
        match ch {
            '\\' if in_string => {
                escape_next = true;
                continue;
            },
            '\'' => {
                if in_string {
                    // Check if next char is also a quote (SQL-style escaping)
                    if let Some(&'\'') = chars.peek() {
                        // Escaped quote - consume the second quote and add a literal quote
                        chars.next(); // consume the second quote
                        current_value.push('\'');
                    } else {
                        // End of string value
                        values.push(SqlValue::String(current_value.clone()));
                        current_value.clear();
                        in_string = false;
                        
                        // Skip to next comma or end
                        while let Some(&next_ch) = chars.peek() {
                            if next_ch == ',' {
                                chars.next(); // consume comma
                                break;
                            } else if next_ch.is_whitespace() {
                                chars.next(); // consume whitespace
                            } else {
                                break;
                            }
                        }
                    }
                } else {
                    // Start of string value
                    in_string = true;
                    // Skip any leading whitespace before string
                    current_value.clear();
                }
            },
            ',' if !in_string => {
                // End of current value
                let trimmed = current_value.trim();
                if !trimmed.is_empty() {
                    values.push(parse_single_value(trimmed)?);
                }
                current_value.clear();
            },
            _ if in_string => {
                current_value.push(ch);
            },
            _ => {
                current_value.push(ch);
            }
        }
    }
    
    // Handle last value
    if in_string {
        return Err("❌ **Unterminated string** - Missing closing quote\n\n**Example:** `'John'` instead of `'John`".to_string());
    }
    
    let trimmed = current_value.trim();
    if !trimmed.is_empty() {
        values.push(parse_single_value(trimmed)?);
    }
    
    if values.is_empty() {
        return Err("❌ **No values provided**\n\n**Examples:**\n• `1, 'John', true`\n• `42, 'Alice', false, NULL`".to_string());
    }
    
    Ok(values)
}

/// Parse a single value (non-string)
fn parse_single_value(value_str: &str) -> Result<SqlValue, String> {
    let trimmed = value_str.trim();
    
    if trimmed.is_empty() {
        return Err("❌ **Empty value** - Use NULL for empty values".to_string());
    }
    
    // Check for NULL
    if trimmed.to_uppercase() == "NULL" {
        return Ok(SqlValue::Null);
    }
    
    // Check for boolean
    match trimmed.to_lowercase().as_str() {
        "true" => return Ok(SqlValue::Boolean(true)),
        "false" => return Ok(SqlValue::Boolean(false)),
        _ => {}
    }
    
    // Try to parse as integer
    if let Ok(int_val) = trimmed.parse::<i64>() {
        return Ok(SqlValue::Integer(int_val));
    }
    
    // Try to parse as float
    if let Ok(float_val) = trimmed.parse::<f64>() {
        return Ok(SqlValue::Float(float_val));
    }
    
    // If all else fails, it's an invalid unquoted value
    Err(format!(
        "❌ **Invalid value:** `{}`\n\n**Valid formats:**\n• Numbers: `42`, `3.14`\n• Booleans: `true`, `false`\n• Strings: `'text'`\n• NULL: `NULL`",
        trimmed
    ))
}

/// Validate SQL values against schema columns
pub fn validate_values_against_schema(values: &[SqlValue], schema: &[ColumnDefinition]) -> Result<(), String> {
    if schema.is_empty() {
        return Ok(()); // No schema to validate against
    }
    
    if values.len() != schema.len() {
        return Err(format!(
            "❌ **Value count mismatch:** Expected {} values for columns, got {}\n\n📋 **Expected columns:** {}\n\n**Example:** {}",
            schema.len(),
            values.len(),
            schema.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", "),
            generate_example_values(schema)
        ));
    }
    
    for (i, (value, column)) in values.iter().zip(schema.iter()).enumerate() {
        if let Err(error) = validate_sql_value_type(value, column, i + 1) {
            return Err(error);
        }
    }
    
    Ok(())
}

/// Validate a single SQL value against a column definition
fn validate_sql_value_type(value: &SqlValue, column: &ColumnDefinition, position: usize) -> Result<(), String> {
    // Check for NULL values
    if matches!(value, SqlValue::Null) {
        if !column.nullable {
            return Err(format!(
                "❌ **NULL not allowed** for column **{}** (position {})\n\n📋 **Column:** {} {}\n**Required:** This column cannot be NULL",
                column.name,
                position,
                column.name,
                column.data_type
            ));
        }
        return Ok(()); // NULL is valid for nullable columns
    }
    
    // Type-specific validation
    match column.data_type.as_str() {
        "INT" => {
            if !matches!(value, SqlValue::Integer(_)) {
                return Err(format!(
                    "❌ **Type mismatch** for column **{}** (position {})\n\nExpected: **integer**\nGot: **{}**\n\n**Example:** `42` instead of `{}`",
                    column.name,
                    position,
                    get_sql_value_type_name(value),
                    value
                ));
            }
        },
        "VARCHAR" | "CHAR" => {
            if let SqlValue::String(s) = value {
                if let Some(max_size) = column.size {
                    if s.len() > max_size as usize {
                        return Err(format!(
                            "❌ **String too long** for column **{}** (position {})\n\nLength: {} characters\nMaximum: {} characters\n\n📏 **Current:** '{}...'\n💡 **Tip:** Shorten the text or increase the column size",
                            column.name,
                            position,
                            s.len(),
                            max_size,
                            &s[..std::cmp::min(20, s.len())]
                        ));
                    }
                }
            } else {
                return Err(format!(
                    "❌ **Type mismatch** for column **{}** (position {})\n\nExpected: **string**\nGot: **{}**\n\n**Example:** `'John'` instead of `{}`",
                    column.name,
                    position,
                    get_sql_value_type_name(value),
                    value
                ));
            }
        },
        "BOOLEAN" => {
            if !matches!(value, SqlValue::Boolean(_)) {
                return Err(format!(
                    "❌ **Type mismatch** for column **{}** (position {})\n\nExpected: **boolean**\nGot: **{}**\n\n**Example:** `true` or `false` instead of `{}`",
                    column.name,
                    position,
                    get_sql_value_type_name(value),
                    value
                ));
            }
        },
        "FLOAT" | "DOUBLE" | "DECIMAL" => {
            if !matches!(value, SqlValue::Float(_) | SqlValue::Integer(_)) {
                return Err(format!(
                    "❌ **Type mismatch** for column **{}** (position {})\n\nExpected: **number**\nGot: **{}**\n\n**Examples:** `3.14` or `42` instead of `{}`",
                    column.name,
                    position,
                    get_sql_value_type_name(value),
                    value
                ));
            }
        },
        "DATE" | "TIME" | "DATETIME" => {
            if let SqlValue::String(s) = value {
                // Validate ISO format for date/time types
                match column.data_type.as_str() {
                    "DATE" => {
                        if !is_valid_iso_date(s) {
                            return Err(format!(
                                "❌ **Invalid DATE format** for column **{}** (position {})\n\nExpected: **ISO 8601 date** (YYYY-MM-DD)\nGot: **'{}'**\n\n**Valid examples:**\n• `'2025-08-19'`\n• `'2023-12-25'`\n• `'2024-02-29'` (leap year)",
                                column.name,
                                position,
                                s
                            ));
                        }
                    },
                    "TIME" => {
                        if !is_valid_iso_time(s) {
                            return Err(format!(
                                "❌ **Invalid TIME format** for column **{}** (position {})\n\nExpected: **ISO 8601 time** (HH:MM:SS[.fraction][Z|±HH:MM])\nGot: **'{}'**\n\n**Valid examples:**\n• `'14:30:00'`\n• `'09:15:30.123'`\n• `'23:59:59Z'`\n• `'12:00:00+02:00'`",
                                column.name,
                                position,
                                s
                            ));
                        }
                    },
                    "DATETIME" => {
                        if !is_valid_iso_datetime(s) {
                            return Err(format!(
                                "❌ **Invalid DATETIME format** for column **{}** (position {})\n\nExpected: **ISO 8601 datetime** (YYYY-MM-DDTHH:MM:SS[.fraction][Z|±HH:MM])\nGot: **'{}'**\n\n**Valid examples:**\n• `'2025-08-19T14:30:00Z'`\n• `'2023-12-25T09:15:30.123Z'`\n• `'2024-06-15T12:00:00+02:00'`\n• `'2025-01-01T00:00:00.000Z'`",
                                column.name,
                                position,
                                s
                            ));
                        }
                    },
                    _ => {}
                }
            } else {
                return Err(format!(
                    "❌ **Type mismatch** for column **{}** (position {})\n\nExpected: **string** (ISO date format)\nGot: **{}**\n\n**Examples:**\n• DATE: `'2023-12-25'`\n• TIME: `'14:30:00'`\n• DATETIME: `'2023-12-25T14:30:00Z'`",
                    column.name,
                    position,
                    get_sql_value_type_name(value)
                ));
            }
        },
        _ => {
            // Unknown type, allow any value
        }
    }
    
    Ok(())
}

/// Get human-readable type name for SQL value
fn get_sql_value_type_name(value: &SqlValue) -> &'static str {
    match value {
        SqlValue::Integer(_) => "integer",
        SqlValue::Float(_) => "number",
        SqlValue::String(_) => "string",
        SqlValue::Boolean(_) => "boolean",
        SqlValue::Null => "null",
    }
}

/// Generate example values for a schema
fn generate_example_values(schema: &[ColumnDefinition]) -> String {
    schema.iter().map(|col| {
        match col.data_type.as_str() {
            "INT" => "42".to_string(),
            "VARCHAR" | "CHAR" => "'text'".to_string(),
            "BOOLEAN" => "true".to_string(),
            "FLOAT" | "DOUBLE" | "DECIMAL" => "3.14".to_string(),
            "DATE" => "'2023-12-25'".to_string(),
            "TIME" => "'14:30:00'".to_string(),
            "DATETIME" => "'2023-12-25T14:30:00Z'".to_string(),
            _ => "'value'".to_string(),
        }
    }).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_column_definitions() {
        let schema = "PersonID int, LastName varchar(255), FirstName varchar(255), Address varchar(255), City varchar(255)";
        let columns = parse_column_definitions(schema).unwrap();
        
        assert_eq!(columns.len(), 5);
        assert_eq!(columns[0].name, "PersonID");
        assert_eq!(columns[0].data_type, "INT");
        assert_eq!(columns[1].name, "LastName");
        assert_eq!(columns[1].data_type, "VARCHAR");
        assert_eq!(columns[1].size, Some(255));
    }

    #[test]
    fn test_varchar_requires_size() {
        let schema = "name VARCHAR";
        let result = parse_column_definitions(schema);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("VARCHAR") && error.contains("requires a size specification"));
    }

    #[test]
    fn test_char_requires_size() {
        let schema = "code CHAR";
        let result = parse_column_definitions(schema);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("CHAR") && error.contains("requires a size specification"));
    }

    #[test]
    fn test_boolean_rejects_size() {
        let schema = "active BOOLEAN(1)";
        let result = parse_column_definitions(schema);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("BOOLEAN") && error.contains("does not support size specification"));
    }

    #[test]
    fn test_date_types_reject_size() {
        let test_cases = vec![
            "birth_date DATE(10)",
            "login_time TIME(8)",
            "created_at DATETIME(20)",
        ];
        
        for case in test_cases {
            let result = parse_column_definitions(case);
            assert!(result.is_err(), "Expected error for: {}", case);
            let error = result.unwrap_err();
            assert!(error.contains("does not support size specification"), "Error should mention size specification for: {}", case);
        }
    }

    #[test]
    fn test_int_rejects_size() {
        let schema = "id INT(11)";
        let result = parse_column_definitions(schema);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("INT") && error.contains("does not support size specification"));
    }

    #[test]
    fn test_decimal_with_valid_size() {
        let schema = "price DECIMAL(10), amount FLOAT(7), total DOUBLE(15)";
        let columns = parse_column_definitions(schema).unwrap();
        
        assert_eq!(columns.len(), 3);
        assert_eq!(columns[0].data_type, "DECIMAL");
        assert_eq!(columns[0].size, Some(10));
        assert_eq!(columns[1].data_type, "FLOAT");
        assert_eq!(columns[1].size, Some(7));
        assert_eq!(columns[2].data_type, "DOUBLE");
        assert_eq!(columns[2].size, Some(15));
    }

    #[test]
    fn test_decimal_with_invalid_size() {
        let test_cases = vec![
            "price DECIMAL(0)",
            "amount FLOAT(66)",
            "total DOUBLE(100)",
        ];
        
        for case in test_cases {
            let result = parse_column_definitions(case);
            assert!(result.is_err(), "Expected error for: {}", case);
        }
    }

    #[test]
    fn test_varchar_size_validation() {
        // Test zero size
        let result = parse_column_definitions("name VARCHAR(0)");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("size must be greater than 0"));
        
        // Test too large size
        let result = parse_column_definitions("description VARCHAR(70000)");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("size") && error.contains("too large"));
        
        // Test valid size
        let result = parse_column_definitions("name VARCHAR(255)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_data_type() {
        let schema = "id INT, name INVALID_TYPE";
        let result = parse_column_definitions(schema);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("INVALID_TYPE"));
        assert!(error.contains("not a valid data type"));
    }

    #[test]
    fn test_parse_sql_values_basic() {
        let input = "1, 'test', true, 3.14, NULL";
        let result = parse_sql_values(input).unwrap();
        
        assert_eq!(result.len(), 5);
        assert!(matches!(result[0], SqlValue::Integer(1)));
        assert!(matches!(result[1], SqlValue::String(ref s) if s == "test"));
        assert!(matches!(result[2], SqlValue::Boolean(true)));
        assert!(matches!(result[3], SqlValue::Float(f) if (f - 3.14).abs() < 0.001));
        assert!(matches!(result[4], SqlValue::Null));
    }

    #[test]
    fn test_parse_sql_values_with_whitespace() {
        let input = " 42 ,  'hello world'  , false ";
        let result = parse_sql_values(input).unwrap();
        
        assert_eq!(result.len(), 3);
        assert!(matches!(result[0], SqlValue::Integer(42)));
        assert!(matches!(result[1], SqlValue::String(ref s) if s == "hello world"));
        assert!(matches!(result[2], SqlValue::Boolean(false)));
    }

    #[test]
    fn test_parse_sql_values_quoted_strings() {
        let input = r#"'simple', 'with ''escaped'' quotes'"#;
        let result = parse_sql_values(input).unwrap();
        
        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], SqlValue::String(ref s) if s == "simple"));
        assert!(matches!(result[1], SqlValue::String(ref s) if s == "with 'escaped' quotes"));
    }

    #[test]
    fn test_parse_sql_values_error_cases() {
        // Unterminated string
        let result = parse_sql_values("1, 'unterminated");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unterminated"));
        
        // Invalid unquoted value
        let result = parse_sql_values("1, abc, 3");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid value"));
    }

    #[test]
    fn test_validate_values_against_schema() {
        let schema = vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INT".to_string(),
                size: None,
                nullable: false,
                primary_key: true,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "VARCHAR".to_string(),
                size: Some(10),
                nullable: false,
                primary_key: false,
            },
        ];
        
        // Valid values
        let values = vec![
            SqlValue::Integer(1),
            SqlValue::String("John".to_string()),
        ];
        assert!(validate_values_against_schema(&values, &schema).is_ok());
        
        // Too few values
        let values = vec![SqlValue::Integer(1)];
        let result = validate_values_against_schema(&values, &schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected 2 values"));
        
        // Wrong type
        let values = vec![
            SqlValue::String("not a number".to_string()),
            SqlValue::String("John".to_string()),
        ];
        let result = validate_values_against_schema(&values, &schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
        
        // String too long
        let values = vec![
            SqlValue::Integer(1),
            SqlValue::String("This string is way too long".to_string()),
        ];
        let result = validate_values_against_schema(&values, &schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("String too long"));
    }
}

/// Validate ISO 8601 date format (YYYY-MM-DD)
fn is_valid_iso_date(date_str: &str) -> bool {
    if date_str.len() != 10 {
        return false;
    }
    
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    
    // Parse year, month, day
    let year = match parts[0].parse::<i32>() {
        Ok(y) if y >= 1000 && y <= 9999 && parts[0].len() == 4 => y,
        _ => return false,
    };
    
    let month = match parts[1].parse::<u32>() {
        Ok(m) if m >= 1 && m <= 12 && parts[1].len() == 2 => m,
        _ => return false,
    };
    
    let day = match parts[2].parse::<u32>() {
        Ok(d) if d >= 1 && d <= 31 && parts[2].len() == 2 => d,
        _ => return false,
    };
    
    // Basic month/day validation
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => day <= 31,
        4 | 6 | 9 | 11 => day <= 30,
        2 => {
            // February leap year check
            let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            day <= if is_leap { 29 } else { 28 }
        },
        _ => false,
    }
}

/// Validate ISO 8601 time format (HH:MM:SS[.fraction][Z|±HH:MM])
fn is_valid_iso_time(time_str: &str) -> bool {
    // Handle timezone suffix
    let (time_part, _tz_part) = if time_str.ends_with('Z') {
        (&time_str[..time_str.len()-1], Some("Z"))
    } else if let Some(pos) = time_str.rfind('+').or_else(|| time_str.rfind('-')) {
        if pos > 6 { // Ensure we don't split on date part
            (&time_str[..pos], Some(&time_str[pos..]))
        } else {
            (time_str, None)
        }
    } else {
        (time_str, None)
    };
    
    // Split main time components
    let main_parts: Vec<&str> = time_part.split(':').collect();
    if main_parts.len() != 3 {
        return false;
    }
    
    // Validate hours
    let _hours = match main_parts[0].parse::<u32>() {
        Ok(h) if h <= 23 && main_parts[0].len() == 2 => h,
        _ => return false,
    };
    
    // Validate minutes
    let _minutes = match main_parts[1].parse::<u32>() {
        Ok(m) if m <= 59 && main_parts[1].len() == 2 => m,
        _ => return false,
    };
    
    // Validate seconds (may include fractional part)
    let seconds_part = main_parts[2];
    if seconds_part.contains('.') {
        let sec_parts: Vec<&str> = seconds_part.split('.').collect();
        if sec_parts.len() != 2 {
            return false;
        }
        
        // Validate whole seconds
        let _seconds = match sec_parts[0].parse::<u32>() {
            Ok(s) if s <= 59 && sec_parts[0].len() == 2 => s,
            _ => return false,
        };
        
        // Validate fractional seconds (must be digits)
        let fraction = sec_parts[1];
        if fraction.is_empty() || !fraction.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    } else {
        // No fractional part
        let _seconds = match seconds_part.parse::<u32>() {
            Ok(s) if s <= 59 && seconds_part.len() == 2 => s,
            _ => return false,
        };
    }
    
    true
}

/// Validate ISO 8601 datetime format (YYYY-MM-DDTHH:MM:SS[.fraction][Z|±HH:MM])
fn is_valid_iso_datetime(datetime_str: &str) -> bool {
    if !datetime_str.contains('T') {
        return false;
    }
    
    let parts: Vec<&str> = datetime_str.split('T').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let date_part = parts[0];
    let time_part = parts[1];
    
    is_valid_iso_date(date_part) && is_valid_iso_time(time_part)
}

#[cfg(test)]
mod iso_tests {
    use super::*;

    #[test]
    fn test_valid_iso_dates() {
        assert!(is_valid_iso_date("2025-08-19"));
        assert!(is_valid_iso_date("2023-12-25"));
        assert!(is_valid_iso_date("2024-02-29")); // leap year
        assert!(is_valid_iso_date("2000-02-29")); // leap year
        assert!(is_valid_iso_date("1999-12-31"));
    }

    #[test]
    fn test_invalid_iso_dates() {
        assert!(!is_valid_iso_date("2023-13-01")); // invalid month
        assert!(!is_valid_iso_date("2023-02-30")); // invalid day for February
        assert!(!is_valid_iso_date("2023-04-31")); // invalid day for April
        assert!(!is_valid_iso_date("2023-2-29")); // non-leap year
        assert!(!is_valid_iso_date("23-08-19")); // wrong year format
        assert!(!is_valid_iso_date("2023/08/19")); // wrong separator
        assert!(!is_valid_iso_date("2023-8-19")); // missing zero padding
        assert!(!is_valid_iso_date("")); // empty string
        assert!(!is_valid_iso_date("not-a-date")); // invalid format
    }

    #[test]
    fn test_valid_iso_times() {
        assert!(is_valid_iso_time("14:30:00"));
        assert!(is_valid_iso_time("09:15:30"));
        assert!(is_valid_iso_time("23:59:59"));
        assert!(is_valid_iso_time("00:00:00"));
        assert!(is_valid_iso_time("12:30:45.123"));
        assert!(is_valid_iso_time("14:30:00Z"));
        assert!(is_valid_iso_time("12:00:00+02:00"));
        assert!(is_valid_iso_time("08:30:15-05:00"));
        assert!(is_valid_iso_time("16:45:30.999Z"));
    }

    #[test]
    fn test_invalid_iso_times() {
        assert!(!is_valid_iso_time("25:30:00")); // invalid hour
        assert!(!is_valid_iso_time("14:60:00")); // invalid minute
        assert!(!is_valid_iso_time("14:30:60")); // invalid second
        assert!(!is_valid_iso_time("14:30")); // missing seconds
        assert!(!is_valid_iso_time("14-30-00")); // wrong separator
        assert!(!is_valid_iso_time("2:30:00")); // missing zero padding
        assert!(!is_valid_iso_time("14:30:00.")); // empty fraction
        assert!(!is_valid_iso_time("")); // empty string
        assert!(!is_valid_iso_time("not-a-time")); // invalid format
    }

    #[test]
    fn test_valid_iso_datetimes() {
        assert!(is_valid_iso_datetime("2025-08-19T14:30:00Z"));
        assert!(is_valid_iso_datetime("2023-12-25T09:15:30.123Z"));
        assert!(is_valid_iso_datetime("2024-06-15T12:00:00+02:00"));
        assert!(is_valid_iso_datetime("2025-01-01T00:00:00.000Z"));
        assert!(is_valid_iso_datetime("2023-02-28T23:59:59"));
    }

    #[test]
    fn test_invalid_iso_datetimes() {
        assert!(!is_valid_iso_datetime("2025-08-19 14:30:00")); // missing T
        assert!(!is_valid_iso_datetime("2025-13-19T14:30:00Z")); // invalid month
        assert!(!is_valid_iso_datetime("2025-08-19T25:30:00Z")); // invalid hour
        assert!(!is_valid_iso_datetime("2025-08-19T14:60:00Z")); // invalid minute
        assert!(!is_valid_iso_datetime("not-a-datetime")); // invalid format
        assert!(!is_valid_iso_datetime("")); // empty string
    }
}
