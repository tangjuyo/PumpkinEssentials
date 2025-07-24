//! Homes command for PumpkinEssentials
//!
//! This command allows players to list all their homes.

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

use crate::managers::home_manager::get_home_manager;

const NAMES: [&str; 1] = ["homes"];
const DESCRIPTION: &str = "List all your homes";

// /homes command
struct HomesExecutor;

#[async_trait]
impl CommandExecutor for HomesExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            let player_uuid = target.gameprofile.id;
            
            // Get home manager
            let home_manager = get_home_manager().await
                .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to get home manager: {}", e)))))?;

            // Get all homes for the player
            let homes = home_manager.get_player_homes(player_uuid).await
                .map_err(|e| CommandError::CommandFailed(Box::new(TextComponent::text(format!("Failed to get homes: {}", e)))))?;

            if homes.is_empty() {
                target.send_system_message(&TextComponent::text("&cYou don't have any homes set.")).await;
                return Ok(());
            }

            // Display homes
            let mut message = format!("&aYou have {} home(s):", homes.len());
            
            for (home_name, position, _, _) in homes {
                message.push_str(&format!(
                    "\n&7- &e{}&7: &f({:.1}, {:.1}, {:.1})",
                    home_name, position.x, position.y, position.z
                ));
            }

            target.send_system_message(&TextComponent::text(message.clone())).await;
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
            .execute(HomesExecutor)
    )
} 