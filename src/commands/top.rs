use async_trait::async_trait;
use pumpkin::command::CommandSender::Player;
use pumpkin::{
    command::{
        args::ConsumedArgs, dispatcher::CommandError, dispatcher::CommandError::InvalidRequirement,
        tree::builder::require, tree::CommandTree, CommandExecutor, CommandSender,
    },
    server::Server,
};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
// TODO: Fix WorldPosition import if needed
// use pumpkin_util::math::position::WorldPosition;

const NAMES: [&str; 1] = ["top"];
const DESCRIPTION: &str = "Teleport to the highest block at your location.";

struct TopExecutor;

#[async_trait]
impl CommandExecutor for TopExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            // TODO: The logic for finding the top block needs to be fixed based on the Pumpkin API.
            // The following code is commented out because it does not compile.

            /*
            let current_pos = target.living_entity.entity.pos.load();
            let world_name = &target.living_entity.entity.world.name;
            let world = server.get_world_by_name(world_name).await.unwrap();

            let mut highest_y = 0.0;

            // Find the highest non-air block
            for y in (0..=320).rev() {
                let block_pos = Vector3::new(current_pos.x.floor() as i32, y, current_pos.z.floor() as i32);
                let chunk_pos = WorldPosition::from_block_pos(block_pos);

                if let Some(chunk) = world.get_chunk(chunk_pos.chunk_x, chunk_pos.chunk_z).await {
                    let block = chunk.get_block(chunk_pos.chunk_x, y, chunk_pos.chunk_z);
                    if block.to_registry_id() != 0 { // Not air
                        highest_y = (y + 1) as f64;
                        break;
                    }
                }
            }

            if highest_y > 0.0 {
                let new_pos = Vector3::new(current_pos.x, highest_y, current_pos.z);
                let yaw = target.living_entity.entity.yaw.load();
                let pitch = target.living_entity.entity.pitch.load();

                target.teleport(new_pos, yaw, pitch).await;

                target
                    .send_system_message(&TextComponent::text(format!(
                        "Teleported to the top ({:.1})",
                        highest_y
                    )))
                    .await;
            } else {
                target
                    .send_system_message(&TextComponent::text("No solid block found below you"))
                    .await;
            }
            */

            target
                .send_system_message(&TextComponent::text(
                    "This command is temporarily disabled.",
                ))
                .await;

            Ok(())
        } else {
            Err(InvalidRequirement)
        }
    }
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(require(|sender| sender.is_player()).execute(TopExecutor))
}
