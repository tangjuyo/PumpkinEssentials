use async_trait::async_trait;
use pumpkin::{
    command::{
        args::{Arg, ConsumedArgs, players::PlayersArgumentConsumer},
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
use crate::{get_fly_state, set_fly_state};

const NAMES: [&str; 1] = ["fly"];
const DESCRIPTION: &str = "Toggle flight mode for yourself or another player.";
const ARG_TARGET: &str = "target";

struct FlyExecutor;

#[async_trait]
impl CommandExecutor for FlyExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            let target_player = if let Some(Arg::Players(players)) = args.get(ARG_TARGET) {
                if players.len() == 1 {
                    players[0].clone()
                } else {
                    return Err(InvalidConsumption(Some("Expected exactly one player".to_string())));
                }
            } else {
                target.clone()
            };

            // Get current fly state from HashMap
            let player_uuid = target_player.gameprofile.id;
            let is_fly_enabled = get_fly_state(player_uuid).await;
            
            // Toggle fly state
            let new_fly_state = !is_fly_enabled;
            set_fly_state(player_uuid, new_fly_state).await;

            // Apply the fly state to the player's abilities
            {
                let mut abilities = target_player.abilities.lock().await;
                if new_fly_state {
                    // Activer le vol : activer allow_flying et flying
                    abilities.allow_flying = true;
                    abilities.flying = true;
                } else {
                    // Désactiver le vol : désactiver flying et allow_flying
                    abilities.flying = false;
                    abilities.allow_flying = false;
                }
            }
            target_player.send_abilities_update().await;

            let player_name = &target_player.gameprofile.name;
            let status = if new_fly_state { "enabled" } else { "disabled" };
            
            if std::ptr::eq(target, &target_player) {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "Flight mode {}",
                        status
                    )))
                    .await;
            } else {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "{} flight mode for {}",
                        if new_fly_state { "Enabled" } else { "Disabled" },
                        player_name
                    )))
                    .await;
                    
                target_player
                    .send_system_message(&TextComponent::text(format!(
                        "Flight mode {}",
                        status
                    )))
                    .await;
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
            .execute(FlyExecutor)
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(FlyExecutor))
    )
}