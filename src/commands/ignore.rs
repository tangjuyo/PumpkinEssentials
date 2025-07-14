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

const NAMES: [&str; 1] = ["ignore"];
const DESCRIPTION: &str = "Ignore a player.";
const ARG_TARGET: &str = "target";

struct IgnoreExecutor;

#[async_trait]
impl CommandExecutor for IgnoreExecutor {
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
                return Err(InvalidConsumption(Some("Target player is required".to_string())));
            };

            // TODO: Implement ignore functionality
            // This would require a system to track ignored players and filter chat messages
            // For now, we'll show a placeholder message
            
            let player_name = &target_player.gameprofile.name;
            
            target
                .send_system_message(&TextComponent::text(format!(
                    "Player ignore functionality is not yet implemented. (Would ignore: {})",
                    player_name
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
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(IgnoreExecutor))
    )
}