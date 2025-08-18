// /sql drop table <name>

use std::error::Error;
use crate::logging::log_info;

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering DROP TABLE command");
    Ok(())
}

pub async fn run(table_name: &str) -> Result<String, String> {
    log_info(&format!("DROP TABLE command executed for table: {}", table_name));
    Ok(format!("Would drop table `{}` (placeholder)", table_name))
}
