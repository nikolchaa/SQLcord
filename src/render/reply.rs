use serenity::builder::CreateInteractionResponseMessage;

pub fn ok_message(content: &str) -> CreateInteractionResponseMessage {
    CreateInteractionResponseMessage::new().content(content)
}
