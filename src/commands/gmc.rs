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
use pumpkin_util::GameMode;

const NAMES: [&str; 1] = ["gmc"];
const DESCRIPTION: &str = "Change your gamemode to creative.";
const ARG_TARGET: &str = "target";

struct GMCExecutor;

#[async_trait]
impl CommandExecutor for GMCExecutor {
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
                target.clone()
            };

            target_player.set_gamemode(GameMode::Creative).await;

            let player_name = &target_player.gameprofile.name;

            if std::ptr::eq(target, &target_player) {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "Set own gamemode to {:?}",
                        GameMode::Creative
                    )))
                    .await;
            } else {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "Set {}'s gamemode to {:?}",
                        player_name,
                        GameMode::Creative
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
            .execute(GMCExecutor)
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(GMCExecutor)),
    )
}
