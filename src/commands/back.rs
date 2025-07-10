use async_trait::async_trait;
use pumpkin::command::CommandSender::Player;
use pumpkin::{
    command::{
        args::ConsumedArgs, dispatcher::CommandError, dispatcher::CommandError::InvalidRequirement,
        tree::builder::require, tree::CommandTree, CommandExecutor, CommandSender,
    },
    plugin::{player::player_teleport::PlayerTeleportEvent, EventHandler},
    server::Server,
};
use pumpkin_api_macros::with_runtime;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// Structure pour stocker la position de retour
#[derive(Clone, Debug)]
pub struct BackLocation {
    pub position: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    pub world_name: String, // Pour les mondes multiples (optionnel pour l'instant)
}

// Global storage for player back locations
lazy_static::lazy_static! {
    pub static ref PLAYER_BACK_LOCATIONS: Arc<Mutex<HashMap<Uuid, BackLocation>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

const NAMES: [&str; 2] = ["back", "return"];
const DESCRIPTION: &str = "Teleport to your last location before teleportation.";

struct BackExecutor;

#[async_trait]
impl CommandExecutor for BackExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            // Check teleport cooldown
            if !crate::can_teleport(target.gameprofile.id).await {
                target
                    .send_system_message(&TextComponent::text(
                        "Please wait before teleporting again",
                    ))
                    .await;
                return Ok(());
            }

            let back_locations = PLAYER_BACK_LOCATIONS.lock().await;

            if let Some(back_location) = back_locations.get(&target.gameprofile.id).cloned() {
                drop(back_locations); // Release the lock before teleporting

                // Validate position before teleporting
                if back_location.position.x.is_finite()
                    && back_location.position.y.is_finite()
                    && back_location.position.z.is_finite()
                    && back_location.yaw.is_finite()
                    && back_location.pitch.is_finite()
                {
                    // Utiliser teleport comme la commande native Pumpkin
                    target
                        .teleport(
                            back_location.position,
                            back_location.yaw,
                            back_location.pitch,
                        )
                        .await;

                    target
                        .send_system_message(&TextComponent::text(
                            "Teleported to your previous location",
                        ))
                        .await;
                } else {
                    target
                        .send_system_message(&TextComponent::text(
                            "Previous location has invalid coordinates",
                        ))
                        .await;
                }

                Ok(())
            } else {
                target
                    .send_system_message(&TextComponent::text("No previous location found"))
                    .await;

                Ok(())
            }
        } else {
            Err(InvalidRequirement)
        }
    }
}

// Event handler for PlayerTeleportEvent to automatically save back locations
pub struct BackLocationHandler;

#[with_runtime(global)]
#[async_trait]
impl EventHandler<PlayerTeleportEvent> for BackLocationHandler {
    async fn handle_blocking(&self, _server: &Arc<Server>, event: &mut PlayerTeleportEvent) {
        // Save the 'from' position as the back location
        let mut back_locations = PLAYER_BACK_LOCATIONS.lock().await;

        let back_location = BackLocation {
            position: event.from,
            yaw: event.player.living_entity.entity.yaw.load(),
            pitch: event.player.living_entity.entity.pitch.load(),
            world_name: "overworld".to_string(), // Default world for now
        };

        back_locations.insert(event.player.gameprofile.id, back_location);
    }
}

// Function to get a player's back location (kept for API compatibility)
pub async fn get_back_location_for_player(player_uuid: Uuid) -> Option<BackLocation> {
    let back_locations = PLAYER_BACK_LOCATIONS.lock().await;
    back_locations.get(&player_uuid).cloned()
}

// Function to clear a player's back location (kept for API compatibility)
pub async fn clear_back_location_for_player(player_uuid: Uuid) {
    let mut back_locations = PLAYER_BACK_LOCATIONS.lock().await;
    back_locations.remove(&player_uuid);
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(require(|sender| sender.is_player()).execute(BackExecutor))
}
