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

const NAMES: [&str; 1] = ["gms"];
const DESCRIPTION: &str = "Change your gamemode to survival.";
const ARG_TARGET: &str = "target";

struct GMSExecutor;

#[async_trait]
impl CommandExecutor for GMSExecutor {
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

            // Vérifier si le joueur est déjà en Survival
            if target_player.gamemode.load() == GameMode::Survival {
                let player_name = &target_player.gameprofile.name;
                if std::ptr::eq(target, &target_player) {
                    target
                        .send_system_message(&TextComponent::text(
                            "You are already in Survival mode."
                        ))
                        .await;
                } else {
                    target
                        .send_system_message(&TextComponent::text(format!(
                            "{} is already in Survival mode.",
                            player_name
                        )))
                        .await;
                }
                return Ok(());
            }

            target_player.set_gamemode(GameMode::Survival).await;

            let player_name = &target_player.gameprofile.name;
            
            if std::ptr::eq(target, &target_player) {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "Set own gamemode to {:?}",
                        GameMode::Survival
                    )))
                    .await;
            } else {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "Set {}'s gamemode to {:?}",
                        player_name,
                        GameMode::Survival
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
            .execute(GMSExecutor)
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(GMSExecutor))
    )
}
