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
}
