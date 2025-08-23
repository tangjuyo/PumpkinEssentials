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
    entity::EntityBase,
};
use pumpkin::command::CommandSender::Player;
use pumpkin_util::text::TextComponent;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use uuid::Uuid;
use crate::managers::spawn_manager::get_spawn_manager;

const NAMES: [&str; 1] = ["spawn"];
const DESCRIPTION: &str = "Teleport to the server spawn point.";
const COOLDOWN: Duration = Duration::from_secs(5);

static SPAWN_COOLDOWNS: once_cell::sync::Lazy<Arc<Mutex<HashMap<Uuid, Instant>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

struct SpawnExecutor;

#[async_trait]
impl CommandExecutor for SpawnExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        _args: &pumpkin::command::args::ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            let uuid = target.gameprofile.id;
            let mut cooldowns = SPAWN_COOLDOWNS.lock().await;
            let now = Instant::now();
            if let Some(last) = cooldowns.get(&uuid) {
                if now.duration_since(*last) < COOLDOWN {
                    let remaining = COOLDOWN - now.duration_since(*last);
                    target
                        .send_system_message(&TextComponent::text(format!(
                            "You must wait {} second{} before using /spawn again.",
                            remaining.as_secs(),
                            if remaining.as_secs() == 1 { "" } else { "s" }
                        )))
                        .await;
                    return Ok(());
                }
            }
            cooldowns.insert(uuid, now);
            drop(cooldowns);

            // Essayer de lire le spawn depuis spawn.yml
            let spawn_manager = get_spawn_manager().await.expect("Spawn manager not initialized");
            if let Some(spawn) = spawn_manager.get_spawn().await {
                let pos = pumpkin_util::math::vector3::Vector3::new(spawn.x, spawn.y, spawn.z);
                let world = target.living_entity.entity.world.clone();
                target.clone().teleport(pos, Some(spawn.yaw), Some(spawn.pitch), world).await;
                target
                    .send_system_message(&TextComponent::text("Teleported to spawn!"))
                    .await;
                return Ok(());
            }

            // Fallback : spawn vanilla
            let world = target.living_entity.entity.world.clone();
            let info = world.level_info.read().await;
            let spawn_x = info.spawn_x;
            let spawn_y = info.spawn_y;
            let spawn_z = info.spawn_z;
            let spawn_yaw = info.spawn_angle;
            let spawn_pitch = 0.0;
            let spawn_pos = pumpkin_util::math::vector3::Vector3::new(spawn_x as f64, spawn_y as f64, spawn_z as f64);
            let world = target.living_entity.entity.world.clone();
            target.clone().teleport(spawn_pos, Some(spawn_yaw), Some(spawn_pitch), world).await;
            target
                .send_system_message(&TextComponent::text("Teleported to spawn! (default)"))
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
        require(|sender| sender.is_player()).execute(SpawnExecutor)
    )
}
