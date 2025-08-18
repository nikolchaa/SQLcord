// /sql insert <table> <data>

use std::error::Error;
use crate::logging::log_info;

pub fn register() -> Result<(), Box<dyn Error>> {
    log_info("Registering INSERT command");
    Ok(())
}

pub async fn run(table_name: &str, data: &str) -> Result<String, String> {
    log_info(&format!("INSERT command executed for table: {} with data: {}", table_name, data));
    Ok(format!("Would insert data into table `{}` (placeholder)", table_name))
}
