use async_trait::async_trait;
use pumpkin::{
    command::{
        args::{Arg, ConsumedArgs, simple::SimpleArgConsumer},
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
use crate::kit_manager::get_kit_manager;

const NAMES: [&str; 1] = ["createkit"];
const DESCRIPTION: &str = "Create a kit from your current inventory.";
const ARG_KIT_NAME: &str = "kit_name";

struct CreateKitExecutor;

#[async_trait]
impl CommandExecutor for CreateKitExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(player) = sender {
            // Récupérer le nom du kit
            let kit_name = if let Some(Arg::Simple(name)) = args.get(ARG_KIT_NAME) {
                name.to_string()
            } else {
                return Err(InvalidConsumption(Some("Kit name is required".to_string())));
            };

            // Valider le nom du kit
            if kit_name.is_empty() {
                return Err(InvalidConsumption(Some("Kit name cannot be empty".to_string())));
            }

            if kit_name.len() > 32 {
                return Err(InvalidConsumption(Some("Kit name is too long (max 32 characters)".to_string())));
            }

            // Vérifier que le nom ne contient que des caractères valides
            if !kit_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                return Err(InvalidConsumption(Some("Kit name can only contain letters, numbers, underscores and hyphens".to_string())));
            }

            // Obtenir le gestionnaire de kits
            let kit_manager = match get_kit_manager().await {
                Ok(manager) => manager,
                Err(_) => {
                    player.send_system_message(&TextComponent::text("Kit system is not available".to_string())).await;
                    return Ok(());
                }
            };

            // Vérifier si le kit existe déjà
            if kit_manager.kit_exists(&kit_name).await.unwrap_or(false) {
                player.send_system_message(&TextComponent::text(format!(
                    "Kit '{}' already exists. Use a different name or delete the existing kit first.",
                    kit_name
                ))).await;
                return Ok(());
            }

            // Créer le kit
            match kit_manager.create_kit(
                kit_name.clone(),
                player.gameprofile.id,
                &*player.inventory,
            ).await {
                Ok(_) => {
                    player.send_system_message(&TextComponent::text(format!(
                        "Kit '{}' created successfully!",
                        kit_name
                    ))).await;
                }
                Err(e) => {
                    player.send_system_message(&TextComponent::text(format!(
                        "Failed to create kit: {}",
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
            .execute(CreateKitExecutor)
            .then(argument(ARG_KIT_NAME, SimpleArgConsumer).execute(CreateKitExecutor))
    )
} 