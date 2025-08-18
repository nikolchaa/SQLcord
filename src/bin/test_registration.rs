use std::error::Error;
use sqlcord::logging::{log_error, log_info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log_info("Testing SQLcord Dynamic Command Registration System");

    // Test the SQL command registration
    match sqlcord::commands::sql::register_all_sql_commands() {
        Ok(()) => {
            log_info("Dynamic command registration test completed successfully!");
            log_info("All commands are ready and registered.");
        }
        Err(e) => {
            log_error(&format!("Dynamic command registration test failed: {}", e));
            return Err(e);
        }
    }
    
    Ok(())
}
