use async_trait::async_trait;
use pumpkin::{
    command::{
        args::{Arg, ConsumedArgs, players::PlayersArgumentConsumer, bounded_num::{BoundedNumArgumentConsumer, ToFromNumber}},
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

const NAMES: [&str; 1] = ["near"];
const DESCRIPTION: &str = "Show nearby players within a specified radius.";
const ARG_TARGET: &str = "target";
const ARG_RADIUS: &str = "radius";

struct NearExecutor;

#[async_trait]
impl CommandExecutor for NearExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(player) = sender {
            // Get the target player (default to sender)
            let target_player = if let Some(Arg::Players(players)) = args.get(ARG_TARGET) {
                if players.len() == 1 {
                    players[0].clone()
                } else {
                    return Err(InvalidConsumption(Some("Expected exactly one player".to_string())));
                }
            } else {
                player.clone()
            };

            // Get the radius (default to 100 blocks)
            let radius = if let Some(Arg::Num(radius_result)) = args.get(ARG_RADIUS) {
                match radius_result {
                    Ok(radius_num) => {
                        if let Some(r) = f64::from_number(&radius_num) {
                            if r <= 0.0 {
                                return Err(InvalidConsumption(Some("Radius must be positive".to_string())));
                            }
                            r
                        } else {
                            return Err(InvalidConsumption(Some("Invalid radius value".to_string())));
                        }
                    }
                    Err(_) => return Err(InvalidConsumption(Some("Invalid radius value".to_string()))),
                }
            } else {
                100.0 // Default radius
            };

            // Get the world and find nearby players
            let world = target_player.living_entity.entity.world.clone();
            let target_pos = target_player.living_entity.entity.pos.load();
            let nearby_players = world.get_nearby_players(target_pos, radius).await;

            // Remove the target player from the list
            let mut filtered_players: Vec<_> = nearby_players
                .into_iter()
                .filter(|(uuid, _)| *uuid != target_player.gameprofile.id)
                .collect();

            // Sort by distance
            filtered_players.sort_by(|a, b| {
                let a_pos = a.1.living_entity.entity.pos.load();
                let b_pos = b.1.living_entity.entity.pos.load();
                let a_dist = a_pos.squared_distance_to_vec(target_pos);
                let b_dist = b_pos.squared_distance_to_vec(target_pos);
                a_dist.partial_cmp(&b_dist).unwrap()
            });

            // Send the result
            if filtered_players.is_empty() {
                let message = if std::ptr::eq(player, &target_player) {
                    format!("No players found within {} blocks of you.", radius)
                } else {
                    format!("No players found within {} blocks of {}.", radius, target_player.gameprofile.name)
                };
                player.send_system_message(&TextComponent::text(message)).await;
            } else {
                let mut message = if std::ptr::eq(player, &target_player) {
                    format!("Players within {} blocks of you:\n", radius)
                } else {
                    format!("Players within {} blocks of {}:\n", radius, target_player.gameprofile.name)
                };

                for (_, nearby_player) in filtered_players {
                    let nearby_pos = nearby_player.living_entity.entity.pos.load();
                    let distance = nearby_pos.squared_distance_to_vec(target_pos).sqrt();
                    message.push_str(&format!("§e{}§r - §a{:.1} blocks§r\n", 
                        nearby_player.gameprofile.name, distance));
                }

                player.send_system_message(&TextComponent::text(message)).await;
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
            .execute(NearExecutor)
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(NearExecutor))
            .then(argument(ARG_RADIUS, BoundedNumArgumentConsumer::<f64>::new().min(0.1)).execute(NearExecutor))
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).then(argument(ARG_RADIUS, BoundedNumArgumentConsumer::<f64>::new().min(0.1)).execute(NearExecutor)))
    )
} 