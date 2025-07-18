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
use pumpkin_util::GameMode;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["gmsp"];
const DESCRIPTION: &str = "Change your gamemode to spectator.";
const ARG_TARGET: &str = "target";

struct GMSPExecutor;

#[async_trait]
impl CommandExecutor for GMSPExecutor {
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

            // Vérifier si le joueur est déjà en Spectator
            if target_player.gamemode.load() == GameMode::Spectator {
                let player_name = &target_player.gameprofile.name;
                if std::ptr::eq(target, &target_player) {
                    target
                        .send_system_message(&TextComponent::text(
                            "You are already in Spectator mode."
                        ))
                        .await;
                } else {
                    target
                        .send_system_message(&TextComponent::text(format!(
                            "{} is already in Spectator mode.",
                            player_name
                        )))
                        .await;
                }
                return Ok(());
            }

            target_player.set_gamemode(GameMode::Spectator).await;

            let player_name = &target_player.gameprofile.name;
            
            if std::ptr::eq(target, &target_player) {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "Set own gamemode to {:?}",
                        GameMode::Spectator
                    )))
                    .await;
            } else {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "Set {}'s gamemode to {:?}",
                        player_name,
                        GameMode::Spectator
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
            .execute(GMSPExecutor)
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(GMSPExecutor))
    )
}
