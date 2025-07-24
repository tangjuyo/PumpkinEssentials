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

const NAMES: [&str; 1] = ["home"];
const DESCRIPTION: &str = "Teleport to your home.";

// /home command
struct HomeExecutor;

#[async_trait]
impl CommandExecutor for HomeExecutor {
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
            let (max_homes, cooldown, cross_world) = config_manager.get_home_config_main().await;
            let detailed_config = config_manager.get_home_config_detailed().await;
            
            // Get messages from configuration
            let success_message = config_manager.get_success_message("home_teleport", "&aWelcome home!").await;
            let error_message = config_manager.get_error_message("home_not_found", "&cHome not found!").await;
            let no_homes_message = config_manager.get_error_message("no_homes", "&cYou have no homes set").await;
            
            // Check if debug mode is enabled
            let debug_enabled = config_manager.is_debug_enabled().await;
            
            if debug_enabled {
                log::info!("[Home Command] Player {} attempting to teleport to home '{}'", 
                    target.gameprofile.name, home_name);
                log::info!("[Home Command] Configuration - Max homes: {}, Cooldown: {}, Cross world: {}", 
                    max_homes, cooldown, cross_world);
            }

            // Check teleport cooldown using configuration
            if !crate::can_teleport(target.gameprofile.id).await {
                let cooldown_message = config_manager.get_error_message("teleport_cooldown", 
                    "&cPlease wait before teleporting again").await;
                target.send_system_message(&TextComponent::text(cooldown_message)).await;
                return Ok(());
            }
            
            // Get home from database
            let home_result = home_manager.get_home(target.gameprofile.id, &home_name).await
                .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to get home: {}", e)))))?;
            
            if let Some((_world_name, position, yaw, pitch)) = home_result {
                // Validate position before teleporting
                if position.x.is_finite() && position.y.is_finite() && position.z.is_finite() 
                    && yaw.is_finite() && pitch.is_finite() {
                    
                    // Utiliser teleport comme la commande native Pumpkin
                    target.teleport(position, yaw, pitch).await;

                    // Use configured success message
                    let formatted_message = success_message.replace("{home_name}", &home_name);
                    target.send_system_message(&TextComponent::text(formatted_message)).await;
                    
                    if debug_enabled {
                        log::info!("[Home Command] Successfully teleported {} to home '{}' at position {:?}", 
                            target.gameprofile.name, home_name, position);
                    }
                } else {
                    let invalid_message = config_manager.get_error_message("invalid_coordinates", 
                        "&cHome has invalid coordinates").await;
                    target.send_system_message(&TextComponent::text(invalid_message)).await;
                }
            } else {
                // Use configured error message
                let formatted_message = error_message.replace("{home_name}", &home_name);
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
            .execute(HomeExecutor)
            .then(argument(ARG_HOME_NAME, SimpleArgConsumer).execute(HomeExecutor))
    )
}
