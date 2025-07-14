use async_trait::async_trait;
use pumpkin::{
    command::{
        args::{Arg, ConsumedArgs, players::PlayersArgumentConsumer, bounded_num::BoundedNumArgumentConsumer, FindArgDefaultName},
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

const NAMES: [&str; 1] = ["speed"];
const DESCRIPTION: &str = "Set walk or fly speed for yourself or another player.";
const ARG_TYPE: &str = "type";
const ARG_SPEED: &str = "speed";
const ARG_TARGET: &str = "target";

fn speed_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::<f32>::new()
        .name("speed")
        .min(0.0)
        .max(10.0)
}

struct SpeedExecutor;

#[async_trait]
impl CommandExecutor for SpeedExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            // Get target player
            let target_player = if let Some(Arg::Players(players)) = args.get(ARG_TARGET) {
                if players.len() == 1 {
                    players[0].clone()
                } else {
                    return Err(InvalidConsumption(Some("Expected exactly one player".to_string())));
                }
            } else {
                target.clone()
            };

            // Get speed type (walk or fly)
            let speed_type = if let Some(Arg::Simple(type_str)) = args.get(ARG_TYPE) {
                match type_str.as_ref() {
                    "walk" => "walk",
                    "fly" => "fly",
                    _ => return Err(InvalidConsumption(Some("Speed type must be 'walk' or 'fly'".to_string()))),
                }
            } else {
                return Err(InvalidConsumption(Some("Speed type is required".to_string())));
            };

            // Get speed value using the consumer method
            let speed = match speed_consumer().find_arg_default_name(args) {
                Ok(Ok(speed)) => speed,
                _ => return Err(InvalidConsumption(Some("Valid speed value is required".to_string()))),
            };

            // Apply speed to player
            {
                let mut abilities = target_player.abilities.lock().await;
                match speed_type {
                    "walk" => {
                        abilities.walk_speed = speed;
                    }
                    "fly" => {
                        abilities.fly_speed = speed;
                    }
                    _ => unreachable!(),
                }
            }
            target_player.send_abilities_update().await;

            let player_name = &target_player.gameprofile.name;
            let speed_type_display = if speed_type == "walk" { "walk" } else { "fly" };
            
            if std::ptr::eq(target, &target_player) {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "{} speed set to {}",
                        speed_type_display,
                        speed
                    )))
                    .await;
            } else {
                target
                    .send_system_message(&TextComponent::text(format!(
                        "{} speed set to {} for {}",
                        speed_type_display,
                        speed,
                        player_name
                    )))
                    .await;
                    
                target_player
                    .send_system_message(&TextComponent::text(format!(
                        "{} speed set to {}",
                        speed_type_display,
                        speed
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
            .then(
                argument(ARG_TYPE, pumpkin::command::args::simple::SimpleArgConsumer)
                    .then(
                        argument(ARG_SPEED, speed_consumer())
                            .execute(SpeedExecutor)
                            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(SpeedExecutor))
                    )
            )
    )
} 