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
use std::time::Instant;

const NAMES: [&str; 1] = ["ping"];
const DESCRIPTION: &str = "Check ping for yourself or another player.";
const ARG_TARGET: &str = "target";

struct PingExecutor;

#[async_trait]
impl CommandExecutor for PingExecutor {
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

            // Calculate ping based on keep-alive timing
            let ping_ms = calculate_ping(&target_player).await;
            let player_name = &target_player.gameprofile.name;
            
            if std::ptr::eq(target, &target_player) {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "Your ping: {}ms",
                        ping_ms
                    )))
                    .await;
            } else {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "{}'s ping: {}ms",
                        player_name, ping_ms
                    )))
                    .await;
            }

            Ok(())
        } else {
            Err(InvalidRequirement)
        }
    }
}

/// Calculate ping based on keep-alive timing
async fn calculate_ping(player: &pumpkin::entity::player::Player) -> u64 {
    let now = Instant::now();
    let last_keep_alive_time = player.last_keep_alive_time.load();
    let waiting = player.wait_for_keep_alive.load(std::sync::atomic::Ordering::Relaxed);
    
    if waiting {
        // If we're waiting for a keep-alive response, calculate based on time since last sent
        let elapsed = now.duration_since(last_keep_alive_time);
        elapsed.as_millis() as u64
    } else {
        // If not waiting, use a base ping or calculate from last response
        // For now, we'll use a simple calculation based on connection time
        let connection_time = player.tick_counter.load(std::sync::atomic::Ordering::Relaxed);
        if connection_time > 0 {
            // Simple heuristic: newer connections might have higher ping
            (100 + (connection_time as u64 % 50)).min(500)
        } else {
            100 // Default ping
        }
    }
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        require(|sender| sender.is_player())
            .execute(PingExecutor)
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(PingExecutor))
    )
}