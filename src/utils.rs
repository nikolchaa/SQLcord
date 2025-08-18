// Small helpers

use serenity::builder::CreateEmbed;
use serenity::model::Color;

/// Sanitize a name for Discord channel usage.
/// Converts to lowercase, replaces spaces with underscores, removes invalid characters.
/// Returns (sanitized_name, was_changed)
pub fn sanitize_channel_name(s: &str) -> (String, bool) {
    let original = s.trim();
    let mut sanitized = original
        .to_lowercase()
        .chars()
        .map(|c| match c {
            ' ' => '_',
            c if c.is_ascii_alphanumeric() || c == '_' => c,
            _ => '_',
        })
        .collect::<String>();
    
    // Replace multiple consecutive underscores with single underscore
    while sanitized.contains("__") {
        sanitized = sanitized.replace("__", "_");
    }
    
    // Trim leading and trailing underscores
    sanitized = sanitized.trim_matches('_').to_string();
    
    let was_changed = original != sanitized;
    (sanitized, was_changed)
}

/// Create a success embed (green color)
pub fn create_success_embed(title: &str, description: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(Color::from_rgb(46, 204, 113)) // Green
        .timestamp(serenity::model::Timestamp::now())
}

/// Create a warning embed (yellow/orange color)
pub fn create_warning_embed(title: &str, description: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(Color::from_rgb(241, 196, 15)) // Yellow
        .timestamp(serenity::model::Timestamp::now())
}

/// Create an error embed (red color)
pub fn create_error_embed(title: &str, description: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(Color::from_rgb(231, 76, 60)) // Red
        .timestamp(serenity::model::Timestamp::now())
}

/// Create an info embed (blue color)
pub fn create_info_embed(title: &str, description: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(Color::from_rgb(52, 152, 219)) // Blue
        .timestamp(serenity::model::Timestamp::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_channel_name() {
        // Test normal case
        assert_eq!(sanitize_channel_name("test"), ("test".to_string(), false));
        
        // Test with spaces
        assert_eq!(sanitize_channel_name("Test test"), ("test_test".to_string(), true));
        
        // Test with uppercase
        assert_eq!(sanitize_channel_name("TestName"), ("testname".to_string(), true));
        
        // Test with special characters (should trim trailing underscores)
        assert_eq!(sanitize_channel_name("test-name!"), ("test_name".to_string(), true));
        
        // Test with underscores (should remain)
        assert_eq!(sanitize_channel_name("test_name"), ("test_name".to_string(), false));
        
        // Test trimming underscores
        assert_eq!(sanitize_channel_name("_test_"), ("test".to_string(), true));
        
        // Test multiple consecutive underscores
        assert_eq!(sanitize_channel_name("test__name"), ("test_name".to_string(), true));
        
        // Test empty after sanitization
        assert_eq!(sanitize_channel_name("___"), ("".to_string(), true));
        
        // Test just numbers
        assert_eq!(sanitize_channel_name("123"), ("123".to_string(), false));
    }
}
