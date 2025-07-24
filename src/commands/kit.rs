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

const NAMES: [&str; 1] = ["kit"];
const DESCRIPTION: &str = "Use a kit or give it to another player.";
const ARG_KIT_NAME: &str = "kit_name";
const ARG_TARGET: &str = "target";

struct KitExecutor;

#[async_trait]
impl CommandExecutor for KitExecutor {
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

            // Déterminer la cible (joueur qui va recevoir le kit)
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

            // Donner le kit
            match kit_manager.give_kit(
                &kit_name,
                target_player.gameprofile.id,
                &*target_player.inventory,
                &target_player.gameprofile.name,
                server,
            ).await {
                Ok(true) => {
                    if std::ptr::eq(player, &target_player) {
                        player.send_system_message(&TextComponent::text(format!(
                            "You received kit '{}'!",
                            kit_name
                        ))).await;
                    } else {
                        player.send_system_message(&TextComponent::text(format!(
                            "Kit '{}' given to {}!",
                            kit_name,
                            target_player.gameprofile.name
                        ))).await;
                        
                        target_player.send_system_message(&TextComponent::text(format!(
                            "You received kit '{}' from {}!",
                            kit_name,
                            player.gameprofile.name
                        ))).await;
                    }
                }
                Ok(false) => {
                    // Kit en cooldown
                    player.send_system_message(&TextComponent::text(format!(
                        "Kit '{}' is on cooldown. Please wait before using it again.",
                        kit_name
                    ))).await;
                }
                Err(e) => {
                    player.send_system_message(&TextComponent::text(format!(
                        "Failed to give kit: {}",
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
            .execute(KitExecutor)
            .then(argument(ARG_KIT_NAME, SimpleArgConsumer).execute(KitExecutor))
            .then(argument(ARG_KIT_NAME, SimpleArgConsumer).then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(KitExecutor)))
    )
} 