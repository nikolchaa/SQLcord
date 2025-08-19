use chrono::Utc;

const GREY: &str = "\x1b[38;2;114;114;114m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[38;2;215;89;89m";
const WHITE: &str = "\x1b[39m";

pub fn log_info(msg: &str) {
    let now = Utc::now();
    let ts = now.format("%Y-%m-%dT%H:%M:%S%.6fZ");
    println!("{}{}{}  {}INFO{} {}",
        GREY, ts, WHITE,
        GREEN, WHITE,
        msg
    );
}

pub fn log_error(msg: &str) {
    let now = Utc::now();
    let ts = now.format("%Y-%m-%dT%H:%M:%S%.6fZ");
    eprintln!("{}{}{} {}ERROR{} {}",
        GREY, ts, WHITE,
        RED, WHITE,
        msg
    );
}
