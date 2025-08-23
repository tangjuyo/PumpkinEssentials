use async_trait::async_trait;
use pumpkin::{
    command::{
        args::ConsumedArgs,
        dispatcher::CommandError,
        dispatcher::CommandError::InvalidRequirement,
        tree::CommandTree,
        tree::builder::require,
        CommandExecutor, CommandSender,
    },
    server::Server,
    entity::EntityBase,
};
use pumpkin::command::CommandSender::Player;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["tpall"];
const DESCRIPTION: &str = "Teleport all players to you.";

struct TpallExecutor;

#[async_trait]
impl CommandExecutor for TpallExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            let target_pos = target.living_entity.entity.pos.load();
            let target_yaw = target.living_entity.entity.yaw.load();
            let target_pitch = target.living_entity.entity.pitch.load();

            let mut teleported_count = 0;
            let players = server.get_all_players().await;

            for player in players {
                if player.gameprofile.id != target.gameprofile.id {
                    // The teleport will automatically trigger the event handler for back location
                    let world = player.living_entity.entity.world.clone();
                    player.clone().teleport(target_pos, Some(target_yaw), Some(target_pitch), world).await;
                    teleported_count += 1;
                }
            }

            target
                .send_system_message(&TextComponent::text(format!(
                    "Teleported {} players to you",
                    teleported_count
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
        require(|sender| sender.is_player()).execute(TpallExecutor)
    )
}
