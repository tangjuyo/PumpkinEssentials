use async_trait::async_trait;
use pumpkin::command::args::simple::SimpleArgConsumer;
use pumpkin::command::args::{Arg, ConsumedArgs};
use pumpkin::command::dispatcher::CommandError;
use pumpkin::command::dispatcher::CommandError::{InvalidConsumption, InvalidRequirement};
use pumpkin::command::tree::builder::{argument, require};
use pumpkin::command::tree::CommandTree;
use pumpkin::command::CommandSender::Player;
use pumpkin::command::{CommandExecutor, CommandSender};
use pumpkin::server::Server;
use pumpkin_util::text::TextComponent;

// Import the global PLAYER_HOMES from home_common.rs
use super::home_common::PLAYER_HOMES;

const NAMES: [&str; 2] = ["renamehome", "rhome"];
const DESCRIPTION: &str = "Rename an existing home.";
const ARG_OLD_NAME: &str = "old_name";
const ARG_NEW_NAME: &str = "new_name";

struct RenameHomeExecutor;

#[async_trait]
impl CommandExecutor for RenameHomeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            let Some(Arg::Simple(old_name)) = args.get(ARG_OLD_NAME) else {
                return Err(InvalidConsumption(Some(ARG_OLD_NAME.into())));
            };

            let Some(Arg::Simple(new_name)) = args.get(ARG_NEW_NAME) else {
                return Err(InvalidConsumption(Some(ARG_NEW_NAME.into())));
            };

            let old_name = old_name.to_string();
            let new_name = new_name.to_string();

            if old_name == new_name {
                target
                    .send_system_message(&TextComponent::text(
                        "Old and new home names cannot be the same",
                    ))
                    .await;
                return Ok(());
            }

            let mut homes = PLAYER_HOMES.lock().await;
            if let Some(player_homes) = homes.get_mut(&target.gameprofile.id) {
                // Check if old home exists
                if let Some(home_data) = player_homes.remove(&old_name) {
                    // Check if new name already exists
                    if player_homes.contains_key(&new_name) {
                        // Put the old home back
                        player_homes.insert(old_name.clone(), home_data);
                        target
                            .send_system_message(&TextComponent::text(format!(
                                "A home named '{}' already exists",
                                new_name
                            )))
                            .await;
                    } else {
                        // Rename the home
                        player_homes.insert(new_name.clone(), home_data);
                        target
                            .send_system_message(&TextComponent::text(format!(
                                "Home '{}' renamed to '{}'",
                                old_name, new_name
                            )))
                            .await;
                    }
                } else {
                    target
                        .send_system_message(&TextComponent::text(format!(
                            "Home '{}' not found",
                            old_name
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
        require(|sender| sender.is_player()).then(
            argument(ARG_OLD_NAME, SimpleArgConsumer)
                .then(argument(ARG_NEW_NAME, SimpleArgConsumer).execute(RenameHomeExecutor)),
        ),
    )
}
