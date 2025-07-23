use async_trait::async_trait;
use pumpkin::{
    command::{
        dispatcher::CommandError,
        dispatcher::CommandError::InvalidRequirement,
        tree::CommandTree,
        tree::builder::require,
        CommandExecutor, CommandSender,
    },
    server::Server,
};
use pumpkin::command::CommandSender::Player;
use pumpkin_util::text::TextComponent;
use crate::spawn_manager::{get_spawn_manager, SpawnLocation};

const NAMES: [&str; 1] = ["setspawn"];
const DESCRIPTION: &str = "Set the server spawn point to your current location.";

struct SetSpawnExecutor;

#[async_trait]
impl CommandExecutor for SetSpawnExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        _args: &pumpkin::command::args::ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            let pos = target.living_entity.entity.pos.load();
            let yaw = target.living_entity.entity.yaw.load();
            let pitch = target.living_entity.entity.pitch.load();
            let world = target.living_entity.entity.world.read().await.clone();
            let world_name = world.level_info.read().await.level_name.clone();
            // Met à jour le LevelData (pour compatibilité vanilla)
            let mut info = world.level_info.write().await;
            info.spawn_x = pos.x as i32;
            info.spawn_y = pos.y as i32;
            info.spawn_z = pos.z as i32;
            info.spawn_angle = yaw;
            drop(info);
            // Sauvegarde dans spawn.yml
            let spawn_manager = get_spawn_manager().await.expect("Spawn manager not initialized");
            let location = SpawnLocation {
                world_name,
                x: pos.x,
                y: pos.y,
                z: pos.z,
                yaw,
                pitch,
            };
            let _ = spawn_manager.set_spawn(location).await;
            target
                .send_system_message(&TextComponent::text("§aSpawn point updated to your location."))
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
        require(|sender| sender.is_player()).execute(SetSpawnExecutor)
    )
}
