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
use crate::home_manager::get_home_manager;

const NAMES: [&str; 1] = ["sethome"];
const DESCRIPTION: &str = "Set your home at your current location.";

// /sethome command
struct SethomeExecutor;

#[async_trait]
impl CommandExecutor for SethomeExecutor {
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
            let (max_homes, _, _) = config_manager.get_home_config_main().await;
            let detailed_config = config_manager.get_home_config_detailed().await;
            
            // Get messages from configuration
            let success_message = config_manager.get_success_message("home_set", "&aHome set successfully!").await;
            let limit_message = config_manager.get_error_message("home_limit", "&cYou have reached the maximum number of homes").await;
            
            // Check if debug mode is enabled
            let debug_enabled = config_manager.is_debug_enabled().await;
            
            if debug_enabled {
                log::info!("[SetHome Command] Player {} attempting to set home '{}'", 
                    target.gameprofile.name, home_name);
                log::info!("[SetHome Command] Configuration - Max homes: {}", max_homes);
            }

            // Check home limit using configuration
            let current_count = home_manager.get_home_count(target.gameprofile.id).await
                .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to get home count: {}", e)))))?;
            
            let home_exists = home_manager.home_exists(target.gameprofile.id, &home_name).await
                .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to check home existence: {}", e)))))?;
            
            if current_count >= max_homes as usize && !home_exists {
                let formatted_message = limit_message.replace("{max_homes}", &max_homes.to_string());
                target.send_system_message(&TextComponent::text(formatted_message)).await;
                return Ok(());
            }
            
            // Get current position
            let position = target.living_entity.entity.pos.load();
            let yaw = target.living_entity.entity.yaw.load();
            let pitch = target.living_entity.entity.pitch.load();
            let world_name = "world".to_string(); // TODO: Get actual world name

            // Check detailed configuration if available
            if let Some((_, _, _, _, _, _min_distance, _max_distance, _allow_combat, _)) = detailed_config {
                // Example: Check if player is in combat (if combat system is available)
                // This is just an example - you would need to implement actual combat detection
                let in_combat = false; // Placeholder - get actual combat state
                
                if !_allow_combat && in_combat {
                    let combat_message = config_manager.get_error_message("in_combat", 
                        "&cYou cannot set a home while in combat").await;
                    target.send_system_message(&TextComponent::text(combat_message)).await;
                    return Ok(());
                }
                
                // Example: Check distance from spawn (if spawn position is available)
                // This is just an example - you would need to get the actual spawn position
                let spawn_distance = 0.0; // Placeholder - get actual spawn distance
                
                if spawn_distance < _min_distance {
                    let distance_message = config_manager.get_error_message("too_close_to_spawn", 
                        "&cYou cannot set a home this close to spawn").await;
                    target.send_system_message(&TextComponent::text(distance_message)).await;
                    return Ok(());
                }
                
                if _max_distance > 0.0 && spawn_distance > _max_distance {
                    let distance_message = config_manager.get_error_message("too_far_from_spawn", 
                        "&cYou cannot set a home this far from spawn").await;
                    target.send_system_message(&TextComponent::text(distance_message)).await;
                    return Ok(());
                }
            }

            // Set the home using the database
            let set_result = home_manager.set_home(
                    target.gameprofile.id,
                    home_name.clone(),
                    world_name,
                    position,
                    yaw,
                    pitch,
                ).await;
            
            if let Err(e) = set_result {
                let error_message = config_manager.get_error_message("home_set_failed", 
                    "&cFailed to set home").await;
                target.send_system_message(&TextComponent::text(error_message)).await;
                return Err(CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to set home: {}", e)))));
            }

            // Use configured success message
            let formatted_message = success_message.replace("{home_name}", &home_name);
            target.send_system_message(&TextComponent::text(formatted_message)).await;
            
            if debug_enabled {
                log::info!("[SetHome Command] Successfully set home '{}' for {} at position {:?}", 
                    home_name, target.gameprofile.name, position);
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
            .execute(SethomeExecutor)
            .then(argument(ARG_HOME_NAME, SimpleArgConsumer).execute(SethomeExecutor))
    )
}
