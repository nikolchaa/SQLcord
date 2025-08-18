// DROP subcommands: db, table

pub mod db;
pub mod table;

use std::error::Error;
use crate::logging::log_info;

/// Register all drop subcommands dynamically
pub fn register_drop_subcommands() -> Result<(), Box<dyn Error>> {
    log_info("Starting dynamic registration of DROP subcommands...");
    
    // Register db command
    log_info("Registering DROP DB command...");
    if let Err(e) = db::register() {
        log_info(&format!("DROP DB command registration failed: {}", e));
        return Err(e);
    }
    log_info("DROP DB command registered successfully");
    
    // Register table command  
    log_info("Registering DROP TABLE command...");
    if let Err(e) = table::register() {
        log_info(&format!("DROP TABLE command registration failed: {}", e));
        return Err(e);
    }
    log_info("DROP TABLE command registered successfully");
    
    log_info("All DROP subcommands registered successfully!");
    Ok(())
}
