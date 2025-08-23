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

const NAMES: [&str; 1] = ["r"];
const DESCRIPTION: &str = "Reply to the last private message you received";

const ARG_MESSAGE: &str = "message";

struct ReplyExecutor;

#[async_trait]
impl CommandExecutor for ReplyExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(player) = sender {
            // Récupérer le message à envoyer
            let message = if let Some(Arg::Simple(msg)) = args.get(ARG_MESSAGE) {
                msg.to_string()
            } else {
                return Err(InvalidConsumption(Some("Message is required".to_string())));
            };

            // Obtenir le gestionnaire de messages privés
            let pm_manager = crate::managers::pm_manager::get_pm_manager().await;
            
            // Récupérer le dernier expéditeur
            let last_sender = pm_manager.get_last_sender(player.gameprofile.id).await;
            
            match last_sender {
                Some(target_uuid) => {
                    // Trouver le joueur cible
                    let target_player = server.get_player_by_uuid(target_uuid).await;
                    
                    match target_player {
                        Some(target) => {
                            // Envoyer le message privé
                            let sender_name = player.gameprofile.name.clone();
                            let target_name = target.gameprofile.name.clone();
                            
                            // Message pour l'expéditeur
                            let sender_msg = TextComponent::text(format!(
                                "&7[&6{} &7-> &6{}&7] &f{}",
                                sender_name, target_name, message
                            ));
                            player.send_system_message(&sender_msg).await;
                            
                            // Message pour le destinataire
                            let target_msg = TextComponent::text(format!(
                                "&7[&6{} &7-> &6{}&7] &f{}",
                                sender_name, target_name, message
                            ));
                            target.send_system_message(&target_msg).await;
                            
                            // Mettre à jour l'historique pour le destinataire
                            pm_manager.set_last_sender(target.gameprofile.id, player.gameprofile.id).await;
                            
                            log::info!("[PM] {} -> {}: {}", sender_name, target_name, message);
                        }
                        None => {
                            player.send_system_message(&TextComponent::text(
                                "&cThe player you're trying to reply to is no longer online.".to_string()
                            )).await;
                        }
                    }
                }
                None => {
                    player.send_system_message(&TextComponent::text(
                        "&cYou have no recent private messages to reply to.".to_string()
                    )).await;
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
            .execute(ReplyExecutor)
            .then(argument(ARG_MESSAGE, SimpleArgConsumer).execute(ReplyExecutor))
    )
} 