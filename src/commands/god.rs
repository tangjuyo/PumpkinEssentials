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

const NAMES: [&str; 1] = ["god"];
const DESCRIPTION: &str = "Toggle god mode for yourself or another player.";
const ARG_TARGET: &str = "target";

struct GodExecutor;

#[async_trait]
impl CommandExecutor for GodExecutor {
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

            // Toggle invulnerability
            let is_invulnerable = target_player.abilities.lock().await.invulnerable;
            {
                let mut abilities = target_player.abilities.lock().await;
                abilities.invulnerable = !is_invulnerable;
            }
            target_player.send_abilities_update().await;

            let player_name = &target_player.gameprofile.name;
            let status = if !is_invulnerable { "enabled" } else { "disabled" };
            
            if std::ptr::eq(target, &target_player) {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "God mode {}",
                        status
                    )))
                    .await;
            } else {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "{} god mode for {}",
                        if !is_invulnerable { "Enabled" } else { "Disabled" },
                        player_name
                    )))
                    .await;
                    
                target_player
                    .send_system_message(&TextComponent::text(format!(
                        "God mode {}",
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
            .execute(GodExecutor)
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(GodExecutor))
    )
}