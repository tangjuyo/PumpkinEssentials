use async_trait::async_trait;
use pumpkin::{
    command::{
        args::{Arg, ConsumedArgs, simple::SimpleArgConsumer},
        dispatcher::CommandError,
        dispatcher::CommandError::InvalidRequirement,
        tree::CommandTree,
        tree::builder::{argument, require},
        CommandExecutor, CommandSender,
    },
    server::Server,
};
use pumpkin::command::CommandSender::Player;
use pumpkin_util::text::TextComponent;

use super::home_common::ARG_HOME_NAME;
use crate::managers::home_manager::get_home_manager;

const NAMES: [&str; 1] = ["delhome"];
const DESCRIPTION: &str = "Delete one of your homes.";

// /delhome command
struct DelhomeExecutor;

#[async_trait]
impl CommandExecutor for DelhomeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            let home_name = if let Some(Arg::Simple(name)) = args.get(ARG_HOME_NAME) {
                name.to_string()
            } else {
                "home".to_string() // default home name
            };

            // Get home manager
            let home_manager = get_home_manager().await
                .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to get home manager: {}", e)))))?;

            // Get configuration
            let config_manager = home_manager.get_config_manager();
            let success_message = config_manager.get_success_message("home_deleted", "&aHome '{}' has been deleted").await;
            let error_message = config_manager.get_error_message("home_not_found", "&cHome '{}' not found").await;
            let no_homes_message = config_manager.get_error_message("no_homes", "&cYou have no homes set").await;

            // Delete home from database
            let deleted = home_manager.delete_home(target.gameprofile.id, &home_name).await
                .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to delete home: {}", e)))))?;

            if deleted {
                let formatted_message = success_message.replace("{}", &home_name);
                target.send_system_message(&TextComponent::text(formatted_message)).await;
            } else {
                // Check if player has any homes at all
                let home_count = home_manager.get_home_count(target.gameprofile.id).await
                    .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to get home count: {}", e)))))?;
                
                if home_count == 0 {
                    target.send_system_message(&TextComponent::text(no_homes_message)).await;
                } else {
                    let formatted_message = error_message.replace("{}", &home_name);
                    target.send_system_message(&TextComponent::text(formatted_message)).await;
                }
            }

            Ok(())
        } else {
            Err(InvalidRequirement)
        }
    }
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        require(|sender| sender.is_player())
            .then(argument(ARG_HOME_NAME, SimpleArgConsumer).execute(DelhomeExecutor))
    )
}
