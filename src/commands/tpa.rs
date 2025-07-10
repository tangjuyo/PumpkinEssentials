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
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// Global storage for teleport requests
lazy_static::lazy_static! {
    pub static ref TELEPORT_REQUESTS: Arc<Mutex<HashMap<Uuid, (Uuid, String)>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

const ARG_TARGET: &str = "target";

// /tpa command
const TPA_NAMES: [&str; 1] = ["tpa"];
const TPA_DESCRIPTION: &str = "Request to teleport to another player.";

struct TpaExecutor;

#[async_trait]
impl CommandExecutor for TpaExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
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
                return Err(InvalidConsumption(Some("Player argument is required".to_string())));
            };

            if target.gameprofile.id == target_player.gameprofile.id {
                target
                    .send_system_message(&TextComponent::text("You cannot teleport to yourself"))
                    .await;
                return Ok(());
            }

            let mut requests = TELEPORT_REQUESTS.lock().await;
            requests.insert(target_player.gameprofile.id, (target.gameprofile.id, "tpa".to_string()));

            target
                .send_system_message(&TextComponent::text(format!(
                    "Teleport request sent to {}",
                    target_player.gameprofile.name
                )))
                .await;

            target_player
                .send_system_message(&TextComponent::text(format!(
                    "{} wants to teleport to you. Use /tpaccept to accept or /tpdeny to deny.",
                    target.gameprofile.name
                )))
                .await;

            Ok(())
        } else {
            Err(InvalidRequirement)
        }
    }
}

// /tpaccept command
const TPACCEPT_NAMES: [&str; 1] = ["tpaccept"];
const TPACCEPT_DESCRIPTION: &str = "Accept a teleport request.";

struct TpacceptExecutor;

#[async_trait]
impl CommandExecutor for TpacceptExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            let mut requests = TELEPORT_REQUESTS.lock().await;
            
            if let Some((requester_id, request_type)) = requests.remove(&target.gameprofile.id) {
                drop(requests);
                
                // Find the requester player
                if let Some(requester) = server.get_player_by_uuid(requester_id).await {
                    let requester_name = requester.gameprofile.name.clone();
                    
                    match request_type.as_str() {
                        "tpa" => {
                            // Check teleport cooldown for requester
                            if !crate::can_teleport(requester.gameprofile.id).await {
                                requester
                                    .send_system_message(&TextComponent::text("Please wait before teleporting again"))
                                    .await;
                                return Ok(());
                            }
                            
                            // Teleport requester to target
                            let target_pos = target.living_entity.entity.pos.load();
                            let target_yaw = target.living_entity.entity.yaw.load();
                            let target_pitch = target.living_entity.entity.pitch.load();
                            
                            // Validate position before teleporting
                            if !pumpkin::world::World::is_valid(target_pos) {
                                requester.send_system_message(&TextComponent::text("Target location is out of world bounds")).await;
                                return Ok(());
                            }
                            if target_pos.x.is_finite() && target_pos.y.is_finite() && target_pos.z.is_finite() 
                                && target_yaw.is_finite() && target_pitch.is_finite() {
                                log::info!("[TPA] Teleporting {} to pos={:?}, yaw={}, pitch={}", requester.gameprofile.name, target_pos, target_yaw, target_pitch);
                                requester.teleport(target_pos, target_yaw, target_pitch).await;
                                
                                requester
                                    .send_system_message(&TextComponent::text(format!(
                                        "Teleported to {}",
                                        target.gameprofile.name
                                    )))
                                    .await;
                            } else {
                                log::warn!("[TPA] Refused teleport for {}: pos={:?}, yaw={}, pitch={}", requester.gameprofile.name, target_pos, target_yaw, target_pitch);
                                requester
                                    .send_system_message(&TextComponent::text("Target location has invalid coordinates"))
                                    .await;
                            }
                        }
                        "tpahere" => {
                            // Check teleport cooldown for target
                            if !crate::can_teleport(target.gameprofile.id).await {
                                target
                                    .send_system_message(&TextComponent::text("Please wait before teleporting again"))
                                    .await;
                                return Ok(());
                            }
                            
                            // Teleport target to requester
                            let requester_pos = requester.living_entity.entity.pos.load();
                            let requester_yaw = requester.living_entity.entity.yaw.load();
                            let requester_pitch = requester.living_entity.entity.pitch.load();
                            
                            // Validate position before teleporting
                            if !pumpkin::world::World::is_valid(requester_pos) {
                                target.send_system_message(&TextComponent::text("Requester location is out of world bounds")).await;
                                return Ok(());
                            }
                            if requester_pos.x.is_finite() && requester_pos.y.is_finite() && requester_pos.z.is_finite() 
                                && requester_yaw.is_finite() && requester_pitch.is_finite() {
                                log::info!("[TPAHERE] Teleporting {} to pos={:?}, yaw={}, pitch={}", target.gameprofile.name, requester_pos, requester_yaw, requester_pitch);
                                target.teleport(requester_pos, requester_yaw, requester_pitch).await;
                                
                                target
                                    .send_system_message(&TextComponent::text(format!(
                                        "Teleported to {}",
                                        requester_name
                                    )))
                                    .await;
                            } else {
                                log::warn!("[TPAHERE] Refused teleport for {}: pos={:?}, yaw={}, pitch={}", target.gameprofile.name, requester_pos, requester_yaw, requester_pitch);
                                target
                                    .send_system_message(&TextComponent::text("Requester location has invalid coordinates"))
                                    .await;
                            }
                        }
                        _ => {}
                    }
                    
                    target
                        .send_system_message(&TextComponent::text(format!(
                            "Teleport request from {} accepted",
                            requester_name
                        )))
                        .await;
                } else {
                    target
                        .send_system_message(&TextComponent::text("The player who requested teleportation is no longer online"))
                        .await;
                }
            } else {
                target
                    .send_system_message(&TextComponent::text("No pending teleport requests"))
                    .await;
            }
            
            Ok(())
        } else {
            Err(InvalidRequirement)
        }
    }
}

// /tpdeny command
const TPDENY_NAMES: [&str; 1] = ["tpdeny"];
const TPDENY_DESCRIPTION: &str = "Deny a teleport request.";

struct TpdenyExecutor;

#[async_trait]
impl CommandExecutor for TpdenyExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            let mut requests = TELEPORT_REQUESTS.lock().await;
            
            if let Some((requester_id, _)) = requests.remove(&target.gameprofile.id) {
                drop(requests);
                
                // Find the requester player to notify them
                if let Some(requester) = server.get_player_by_uuid(requester_id).await {
                    let requester_name = requester.gameprofile.name.clone();
                    
                    requester
                        .send_system_message(&TextComponent::text(format!(
                            "{} denied your teleport request",
                            target.gameprofile.name
                        )))
                        .await;
                    
                    target
                        .send_system_message(&TextComponent::text(format!(
                            "Teleport request from {} denied",
                            requester_name
                        )))
                        .await;
                } else {
                    target
                        .send_system_message(&TextComponent::text("Teleport request denied"))
                        .await;
                }
            } else {
                target
                    .send_system_message(&TextComponent::text("No pending teleport requests"))
                    .await;
            }
            
            Ok(())
        } else {
            Err(InvalidRequirement)
        }
    }
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_tpa_command_tree() -> CommandTree {
    CommandTree::new(TPA_NAMES, TPA_DESCRIPTION).then(
        require(|sender| sender.is_player())
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(TpaExecutor))
    )
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_tpaccept_command_tree() -> CommandTree {
    CommandTree::new(TPACCEPT_NAMES, TPACCEPT_DESCRIPTION).then(
        require(|sender| sender.is_player()).execute(TpacceptExecutor)
    )
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_tpdeny_command_tree() -> CommandTree {
    CommandTree::new(TPDENY_NAMES, TPDENY_DESCRIPTION).then(
        require(|sender| sender.is_player()).execute(TpdenyExecutor)
    )
}
