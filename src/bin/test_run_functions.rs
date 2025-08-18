use std::error::Error;
use sqlcord::logging::{log_error, log_info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log_info("Testing SQLcord Run Functions");

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
