// Admin commands group
pub mod perms;
pub mod debug;

pub fn register_admin_group() -> serenity::builder::CreateCommand {
	use serenity::builder::CreateCommand;
	CreateCommand::new("admin").description("Admin helpers")
}
