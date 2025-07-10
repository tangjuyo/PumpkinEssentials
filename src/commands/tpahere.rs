use async_trait::async_trait;
use pumpkin::command::CommandSender::Player;
use pumpkin::{
    command::{
        args::{players::PlayersArgumentConsumer, Arg, ConsumedArgs},
        dispatcher::CommandError,
        dispatcher::CommandError::{InvalidConsumption, InvalidRequirement},
        tree::builder::{argument, require},
        tree::CommandTree,
        CommandExecutor, CommandSender,
    },
    server::Server,
};
use pumpkin_util::text::TextComponent;

use super::tpa::TELEPORT_REQUESTS;

const NAMES: [&str; 1] = ["tpahere"];
const DESCRIPTION: &str = "Request another player to teleport to you.";
const ARG_TARGET: &str = "target";

struct TpahereExecutor;

#[async_trait]
impl CommandExecutor for TpahereExecutor {
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
                    return Err(InvalidConsumption(Some(
                        "Expected exactly one player".to_string(),
                    )));
                }
            } else {
                return Err(InvalidConsumption(Some(
                    "Player argument is required".to_string(),
                )));
            };

            if target.gameprofile.id == target_player.gameprofile.id {
                target
                    .send_system_message(&TextComponent::text("You cannot teleport to yourself."))
                    .await;
                return Ok(());
            }

            let mut requests = TELEPORT_REQUESTS.lock().await;
            requests.insert(
                target_player.gameprofile.id,
                (target.gameprofile.id, "tpahere".to_string()),
            );

            target
                .send_system_message(&TextComponent::text(format!(
                    "Teleport request sent to {}",
                    target_player.gameprofile.name
                )))
                .await;

            target_player
                .send_system_message(&TextComponent::text(format!(
                    "{} wants you to teleport to them. Use /tpaccept to accept or /tpdeny to deny.",
                    target.gameprofile.name
                )))
                .await;

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
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(TpahereExecutor)),
    )
}
