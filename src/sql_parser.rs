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

        // Set default sizes for types that commonly have them
        if size.is_none() {
            size = match normalized_type.as_str() {
                "VARCHAR" => Some(255), // Default VARCHAR size
                "CHAR" => Some(1),      // Default CHAR size
                _ => None,
            };
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
        return Err("❌ **Unterminated string** - Missing closing quote\n\n💡 **Example:** `'John'` instead of `'John`".to_string());
    }
    
    let trimmed = current_value.trim();
    if !trimmed.is_empty() {
        values.push(parse_single_value(trimmed)?);
    }
    
    if values.is_empty() {
        return Err("❌ **No values provided**\n\n💡 **Examples:**\n• `1, 'John', true`\n• `42, 'Alice', false, NULL`".to_string());
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
        "❌ **Invalid value:** `{}`\n\n💡 **Valid formats:**\n• Numbers: `42`, `3.14`\n• Booleans: `true`, `false`\n• Strings: `'text'`\n• NULL: `NULL`",
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
            "❌ **Value count mismatch:** Expected {} values for columns, got {}\n\n📋 **Expected columns:** {}\n\n💡 **Example:** {}",
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
                "❌ **NULL not allowed** for column **{}** (position {})\n\n📋 **Column:** {} {}\n💡 **Required:** This column cannot be NULL",
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
                    "❌ **Type mismatch** for column **{}** (position {})\n\nExpected: **integer**\nGot: **{}**\n\n💡 **Example:** `42` instead of `{}`",
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
                    "❌ **Type mismatch** for column **{}** (position {})\n\nExpected: **string**\nGot: **{}**\n\n💡 **Example:** `'John'` instead of `{}`",
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
                    "❌ **Type mismatch** for column **{}** (position {})\n\nExpected: **boolean**\nGot: **{}**\n\n💡 **Example:** `true` or `false` instead of `{}`",
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
                    "❌ **Type mismatch** for column **{}** (position {})\n\nExpected: **number**\nGot: **{}**\n\n💡 **Examples:** `3.14` or `42` instead of `{}`",
                    column.name,
                    position,
                    get_sql_value_type_name(value),
                    value
                ));
            }
        },
        "DATE" | "TIME" | "DATETIME" => {
            if !matches!(value, SqlValue::String(_)) {
                return Err(format!(
                    "❌ **Type mismatch** for column **{}** (position {})\n\nExpected: **string** (ISO date format)\nGot: **{}**\n\n💡 **Examples:**\n• DATE: `'2023-12-25'`\n• TIME: `'14:30:00'`\n• DATETIME: `'2023-12-25T14:30:00Z'`",
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
    fn test_varchar_default_size() {
        let schema = "name VARCHAR, description TEXT";
        let columns = parse_column_definitions(schema).unwrap();
        
        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0].name, "name");
        assert_eq!(columns[0].data_type, "VARCHAR");
        assert_eq!(columns[0].size, Some(255)); // Default size
        assert_eq!(columns[1].name, "description");
        assert_eq!(columns[1].data_type, "VARCHAR"); // TEXT normalizes to VARCHAR
        assert_eq!(columns[1].size, Some(255)); // Default size
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
