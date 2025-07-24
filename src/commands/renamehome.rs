use async_trait::async_trait;
use pumpkin::command::args::simple::SimpleArgConsumer;
use pumpkin::command::args::{Arg, ConsumedArgs};
use pumpkin::command::dispatcher::CommandError;
use pumpkin::command::dispatcher::CommandError::{InvalidConsumption, InvalidRequirement};
use pumpkin::command::tree::CommandTree;
use pumpkin::command::tree::builder::{argument, require};
use pumpkin::command::{CommandExecutor, CommandSender};
use pumpkin::command::CommandSender::Player;
use pumpkin::server::Server;
use pumpkin_util::text::TextComponent;
use pumpkin_util::math::vector3::Vector3;

use crate::managers::home_manager::get_home_manager;

const NAMES: [&str; 2] = ["renamehome", "rhome"];
const DESCRIPTION: &str = "Rename an existing home.";
const ARG_OLD_NAME: &str = "old_name";
const ARG_NEW_NAME: &str = "new_name";

struct RenameHomeExecutor;

#[async_trait]
impl CommandExecutor for RenameHomeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            let Some(Arg::Simple(old_name)) = args.get(ARG_OLD_NAME) else {
                return Err(InvalidConsumption(Some(ARG_OLD_NAME.into())));
            };

            let Some(Arg::Simple(new_name)) = args.get(ARG_NEW_NAME) else {
                return Err(InvalidConsumption(Some(ARG_NEW_NAME.into())));
            };

            let old_name = old_name.to_string();
            let new_name = new_name.to_string();

            if old_name == new_name {
                target
                    .send_system_message(&TextComponent::text("Old and new home names cannot be the same"))
                    .await;
                return Ok(());
            }

            // Get home manager
            let home_manager = get_home_manager().await
                .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to get home manager: {}", e)))))?;

            // Get configuration
            let config_manager = home_manager.get_config_manager();
            let success_message = config_manager.get_success_message("home_renamed", "&aHome '{}' renamed to '{}'").await;
            let error_message = config_manager.get_error_message("home_not_found", "&cHome '{}' not found").await;
            let exists_message = config_manager.get_error_message("home_exists", "&cA home named '{}' already exists").await;
            let no_homes_message = config_manager.get_error_message("no_homes", "&cYou have no homes set").await;

            // Check if old home exists
            let old_home_exists = home_manager.home_exists(target.gameprofile.id, &old_name).await
                .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to check old home: {}", e)))))?;

            if !old_home_exists {
                // Check if player has any homes at all
                let home_count = home_manager.get_home_count(target.gameprofile.id).await
                    .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to get home count: {}", e)))))?;
                
                if home_count == 0 {
                    target.send_system_message(&TextComponent::text(no_homes_message)).await;
                } else {
                    let formatted_message = error_message.replace("{}", &old_name);
                    target.send_system_message(&TextComponent::text(formatted_message)).await;
                }
                return Ok(());
            }

            // Check if new name already exists
            let new_home_exists = home_manager.home_exists(target.gameprofile.id, &new_name).await
                .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to check new home: {}", e)))))?;

            if new_home_exists {
                let formatted_message = exists_message.replace("{}", &new_name);
                target.send_system_message(&TextComponent::text(formatted_message)).await;
                return Ok(());
            }

            // Get the old home data
            let old_home_info = home_manager.get_home(target.gameprofile.id, &old_name).await
                .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to get home info: {}", e)))))?;

            if let Some((world_name, position, yaw, pitch)) = old_home_info {
                // Delete the old home
                let deleted = home_manager.delete_home(target.gameprofile.id, &old_name).await
                    .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to delete old home: {}", e)))))?;

                if !deleted {
                    let formatted_message = error_message.replace("{}", &old_name);
                    target.send_system_message(&TextComponent::text(formatted_message)).await;
                    return Ok(());
                }

                // Create the new home with the same data
                let set_result = home_manager.set_home(
                    target.gameprofile.id,
                    new_name.clone(),
                    world_name.clone(),
                    position,
                    yaw,
                    pitch,
                ).await;
                
                if let Err(e) = set_result {
                    // Try to restore the old home if setting the new one failed
                    let _ = home_manager.set_home(
                        target.gameprofile.id,
                        old_name.clone(),
                        world_name.clone(),
                        position,
                        yaw,
                        pitch,
                    ).await;
                    
                    return Err(CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to rename home: {}", e)))));
                }

                // Success message
                let formatted_message = success_message.replace("{}", &old_name).replace("{}", &new_name);
                target.send_system_message(&TextComponent::text(formatted_message)).await;
            } else {
                let formatted_message = error_message.replace("{}", &old_name);
                target.send_system_message(&TextComponent::text(formatted_message)).await;
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
            .then(argument(ARG_OLD_NAME, SimpleArgConsumer)
                .then(argument(ARG_NEW_NAME, SimpleArgConsumer).execute(RenameHomeExecutor)))
    )
} 