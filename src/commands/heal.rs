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

const NAMES: [&str; 1] = ["heal"];
const DESCRIPTION: &str = "Heal yourself or another player.";
const ARG_TARGET: &str = "target";

struct HealExecutor;

#[async_trait]
impl CommandExecutor for HealExecutor {
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

            // Set player's health to maximum (20.0)
            target_player.set_health(20.0).await;

            let player_name = &target_player.gameprofile.name;
            
            if std::ptr::eq(target, &target_player) {
                target
                    .send_system_message(&TextComponent::text("You have been healed!"))
                    .await;
            } else {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "Healed {}",
                        player_name
                    )))
                    .await;
                    
                target_player
                    .send_system_message(&TextComponent::text("You have been healed!"))
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
            .execute(HealExecutor)
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(HealExecutor))
    )
}