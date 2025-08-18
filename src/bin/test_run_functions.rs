use std::error::Error;
use sqlcord::logging::{log_error, log_info};
use sqlcord::sql_parser::{parse_column_definitions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log_info("Testing SQLcord Run Functions");

    // Test CREATE TABLE schema parsing functionality
    log_info("=== Testing CREATE TABLE Schema Parsing ===");
    
    // Test 1: Basic column definitions
    log_info("Test 1: Basic column definitions");
    match parse_column_definitions("id INT, name VARCHAR(255), active BOOLEAN") {
        Ok(columns) => {
            log_info(&format!("✅ SUCCESS: Parsed {} columns", columns.len()));
            for col in &columns {
                log_info(&format!("  - {}", col));
            }
        },
        Err(e) => log_error(&format!("❌ FAILED: {}", e)),
    }

    // Test 2: Column definitions with constraints
    log_info("Test 2: Column definitions with constraints");
    match parse_column_definitions("user_id INT PRIMARY KEY, email VARCHAR(100) NOT NULL, created_at DATETIME") {
        Ok(columns) => {
            log_info(&format!("✅ SUCCESS: Parsed {} columns with constraints", columns.len()));
            for col in &columns {
                log_info(&format!("  - {}", col));
            }
        },
        Err(e) => log_error(&format!("❌ FAILED: {}", e)),
    }

    // Test 3: Default VARCHAR size
    log_info("Test 3: Default VARCHAR size (no explicit size)");
    match parse_column_definitions("name VARCHAR, description TEXT") {
        Ok(columns) => {
            log_info("✅ SUCCESS: VARCHAR defaults applied");
            for col in &columns {
                log_info(&format!("  - {} (size: {:?})", col, col.size));
            }
        },
        Err(e) => log_error(&format!("❌ FAILED: {}", e)),
    }

    // Test 4: Invalid data type (should fail)
    log_info("Test 4: Invalid data type (should fail)");
    match parse_column_definitions("id INT, name INVALID_TYPE") {
        Ok(_) => log_error("❌ FAILED: Should have rejected invalid type"),
        Err(e) => log_info(&format!("✅ SUCCESS: Correctly rejected invalid type\n  Error: {}", e)),
    }

    // Test 5: Complex schema with mixed types
    log_info("Test 6: Complex schema with mixed types");
    match parse_column_definitions("order_id INT PRIMARY KEY, customer_email VARCHAR(150) NOT NULL, order_total DECIMAL, created_date DATE, is_shipped BOOLEAN") {
        Ok(columns) => {
            log_info(&format!("✅ SUCCESS: Parsed complex schema with {} columns", columns.len()));
            for col in &columns {
                log_info(&format!("  - {}", col));
            }
        },
        Err(e) => log_error(&format!("❌ FAILED: {}", e)),
    }

    log_info("=== CREATE TABLE Schema Testing Complete ===\n");

    // Test CREATE TABLE run function - DISABLED: requires Discord context
    log_info("Testing CREATE TABLE command - SKIPPED (requires Discord context)");
    // match sqlcord::commands::sql::create::table::run(&ctx, guild_id, user_id, "users").await {
    //     Ok(result) => log_info(&format!("SUCCESS: {}", result)),
    //     Err(e) => log_error(&format!("{}", e)),
    // }
    
    // Test SELECT run function
    log_info("Testing SELECT command");
    match sqlcord::commands::sql::select::run("users").await {
        Ok(result) => log_info(&format!("SUCCESS: {}", result)),
        Err(e) => log_error(&format!("{}", e)),
    }
    
    // Test INSERT run function
    log_info("Testing INSERT command");
    match sqlcord::commands::sql::insert::run("users", r#"{"name": "John", "age": 30}"#).await {
        Ok(result) => log_info(&format!("SUCCESS: {}", result)),
        Err(e) => log_error(&format!("{}", e)),
    }
    
    // Test UPDATE run function
    log_info("Testing UPDATE command");
    match sqlcord::commands::sql::update::run("users").await {
        Ok(result) => log_info(&format!("SUCCESS: {}", result)),
        Err(e) => log_error(&format!("{}", e)),
    }
    
    // Test DELETE run function
    log_info("Testing DELETE command");
    match sqlcord::commands::sql::delete::run("users").await {
        Ok(result) => log_info(&format!("SUCCESS: {}", result)),
        Err(e) => log_error(&format!("{}", e)),
    }
    
    // Test EXPLAIN run function
    log_info("Testing EXPLAIN command");
    match sqlcord::commands::sql::explain::run("SELECT * FROM users").await {
        Ok(_embed) => log_info("SUCCESS: EXPLAIN returned info embed"),
        Err(_embed) => {
            log_error("EXPLAIN returned error embed");
            return Err("EXPLAIN run returned an error".into());
        }
    }
    
    // Test DROP TABLE run function - DISABLED: requires Discord context
    log_info("Testing DROP TABLE command - SKIPPED (requires Discord context)");
    // match sqlcord::commands::sql::drop::table::run(&ctx, guild_id, user_id, "users").await {
    //     Ok(result) => log_info(&format!("SUCCESS: {}", result)),
    //     Err(e) => log_error(&format!("{}", e)),
    // }
    
    log_info("All run function tests completed!");
    
    Ok(())
}
