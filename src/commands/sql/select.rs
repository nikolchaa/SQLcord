// /sql select <table> [where]

use std::error::Error;
use crate::logging::log_info;

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering SELECT command");
    Ok(())
}

pub async fn run(table_name: &str) -> Result<String, String> {
    log_info(&format!("SELECT command executed for table: {}", table_name));
    Ok(format!("Would select from table `{}` (placeholder)", table_name))
}
