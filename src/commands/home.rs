use async_trait::async_trait;
use pumpkin::command::CommandSender::Player;
use pumpkin::{
    command::{
        args::{simple::SimpleArgConsumer, Arg, ConsumedArgs},
        dispatcher::CommandError,
        dispatcher::CommandError::InvalidRequirement,
        tree::builder::{argument, require},
        tree::CommandTree,
        CommandExecutor, CommandSender,
    },
    server::Server,
};
use pumpkin_util::text::TextComponent;

use super::home_common::{ARG_HOME_NAME, PLAYER_HOMES};

const NAMES: [&str; 1] = ["home"];
const DESCRIPTION: &str = "Teleport to your home.";

// /home command
struct HomeExecutor;

#[async_trait]
impl CommandExecutor for HomeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            let home_name = if let Some(Arg::Simple(name)) = args.get(ARG_HOME_NAME) {
                name.to_string()
            } else {
                "home".to_string() // default home name
            };

            // Check teleport cooldown
            if !crate::can_teleport(target.gameprofile.id).await {
                target
                    .send_system_message(&TextComponent::text(
                        "Please wait before teleporting again",
                    ))
                    .await;
                return Ok(());
            }

            let homes = PLAYER_HOMES.lock().await;
            if let Some(player_homes) = homes.get(&target.gameprofile.id) {
                if let Some((position, yaw, pitch)) = player_homes.get(&home_name) {
                    // Validate position before teleporting
                    if position.x.is_finite()
                        && position.y.is_finite()
                        && position.z.is_finite()
                        && yaw.is_finite()
                        && pitch.is_finite()
                    {
                        // Utiliser teleport comme la commande native Pumpkin
                        target.teleport(*position, *yaw, *pitch).await;

                        target
                            .send_system_message(&TextComponent::text(format!(
                                "Teleported to home '{}'",
                                home_name
                            )))
                            .await;
                    } else {
                        target
                            .send_system_message(&TextComponent::text(format!(
                                "Home '{}' has invalid coordinates",
                                home_name
                            )))
                            .await;
                    }
                } else {
                    target
                        .send_system_message(&TextComponent::text(format!(
                            "Home '{}' not found",
                            home_name
                        )))
                        .await;
                }
            } else {
                target
                    .send_system_message(&TextComponent::text("You have no homes set"))
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
            .execute(HomeExecutor)
            .then(argument(ARG_HOME_NAME, SimpleArgConsumer).execute(HomeExecutor)),
    )
}
