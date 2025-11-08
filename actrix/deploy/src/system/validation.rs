//! Input validation utilities

use regex::Regex;

/// Validate port number
pub fn validate_port(port: u16) -> bool {
    port > 0
}

/// Validate username format
pub fn validate_username(username: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9_-]{1,32}$").unwrap();
    re.is_match(username)
}
