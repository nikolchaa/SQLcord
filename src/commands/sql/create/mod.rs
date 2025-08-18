// /sql create subcommands
pub mod db;
pub mod table;

use std::error::Error;
use crate::logging::log_info;

/// Register all create subcommands dynamically
pub fn register_create_subcommands() -> Result<(), Box<dyn Error>> {
    log_info("Starting dynamic registration of CREATE subcommands...");
    
    // Register db command
    log_info("Registering DB command...");
    if let Err(e) = db::register() {
        log_info(&format!("DB command registration failed: {}", e));
        return Err(e);
    }
    log_info("DB command registered successfully");
    
    // Register table command  
    log_info("Registering TABLE command...");
    if let Err(e) = table::register() {
        log_info(&format!("TABLE command registration failed: {}", e));
        return Err(e);
    }
    log_info("TABLE command registered successfully");
    
    log_info("All CREATE subcommands registered successfully!");
    Ok(())
}
