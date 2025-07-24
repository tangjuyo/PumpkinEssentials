use async_trait::async_trait;
use pumpkin::{
    command::{
        args::{Arg, ConsumedArgs, simple::SimpleArgConsumer, players::PlayersArgumentConsumer},
        dispatcher::CommandError,
        dispatcher::CommandError::{InvalidConsumption, InvalidRequirement},
        tree::CommandTree,
        tree::builder::{argument, require},
        CommandExecutor, CommandSender,
    },
    server::Server,
};
use pumpkin::command::CommandSender::Player;
use pumpkin_util::text::TextComponent;
use crate::managers::kit_manager::get_kit_manager;

const NAMES: [&str; 1] = ["resetkit"];
const DESCRIPTION: &str = "Reset the cooldown of a kit for a player";

const ARG_KIT_NAME: &str = "kit_name";
const ARG_TARGET: &str = "target";

struct ResetKitExecutor;

#[async_trait]
impl CommandExecutor for ResetKitExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(player) = sender {
            // Récupérer le nom du kit
            let kit_name = if let Some(Arg::Simple(name)) = args.get(ARG_KIT_NAME) {
                name.to_string()
            } else {
                return Err(InvalidConsumption(Some("Kit name is required".to_string())));
            };

            // Déterminer la cible (joueur dont le cooldown sera reset)
            let target_player = if let Some(Arg::Players(players)) = args.get(ARG_TARGET) {
                if players.len() == 1 {
                    players[0].clone()
                } else {
                    return Err(InvalidConsumption(Some("Expected exactly one player".to_string())));
                }
            } else {
                player.clone() // Par défaut, le joueur qui exécute la commande
            };

            // Obtenir le gestionnaire de kits
            let kit_manager = match get_kit_manager().await {
                Ok(manager) => manager,
                Err(_) => {
                    player.send_system_message(&TextComponent::text("Kit system is not available".to_string())).await;
                    return Ok(());
                }
            };

            // Vérifier si le kit existe
            if !kit_manager.kit_exists(&kit_name).await.unwrap_or(false) {
                player.send_system_message(&TextComponent::text(format!(
                    "Kit '{}' does not exist.",
                    kit_name
                ))).await;
                return Ok(());
            }

            // Reset le cooldown du kit
            match kit_manager.reset_kit_cooldown(
                &kit_name,
                target_player.gameprofile.id,
            ).await {
                Ok(true) => {
                    if std::ptr::eq(player, &target_player) {
                        player.send_system_message(&TextComponent::text(format!(
                            "Cooldown for kit '{}' has been reset!",
                            kit_name
                        ))).await;
                    } else {
                        player.send_system_message(&TextComponent::text(format!(
                            "Cooldown for kit '{}' has been reset for {}!",
                            kit_name,
                            target_player.gameprofile.name
                        ))).await;
                        
                        target_player.send_system_message(&TextComponent::text(format!(
                            "Your cooldown for kit '{}' has been reset by {}!",
                            kit_name,
                            player.gameprofile.name
                        ))).await;
                    }
                }
                Ok(false) => {
                    player.send_system_message(&TextComponent::text(format!(
                        "No cooldown found for kit '{}' for {}.",
                        kit_name,
                        target_player.gameprofile.name
                    ))).await;
                }
                Err(e) => {
                    player.send_system_message(&TextComponent::text(format!(
                        "Failed to reset kit cooldown: {}",
                        e
                    ))).await;
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
            .execute(ResetKitExecutor)
            .then(argument(ARG_KIT_NAME, SimpleArgConsumer).execute(ResetKitExecutor))
            .then(argument(ARG_KIT_NAME, SimpleArgConsumer).then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(ResetKitExecutor)))
    )
} 