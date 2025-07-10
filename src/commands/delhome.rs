use async_trait::async_trait;
use pumpkin::{
    command::{
        args::{Arg, ConsumedArgs, simple::SimpleArgConsumer},
        dispatcher::CommandError,
        dispatcher::CommandError::InvalidRequirement,
        tree::CommandTree,
        tree::builder::{argument, require},
        CommandExecutor, CommandSender,
    },
    server::Server,
};
use pumpkin::command::CommandSender::Player;

use super::home_common::{PLAYER_HOMES, ARG_HOME_NAME};

const NAMES: [&str; 1] = ["delhome"];
const DESCRIPTION: &str = "Delete one of your homes.";

// /delhome command
struct DelhomeExecutor;

#[async_trait]
impl CommandExecutor for DelhomeExecutor {
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

            let mut homes = PLAYER_HOMES.lock().await;
            
            if let Some(player_homes) = homes.get_mut(&target.gameprofile.id) {
                if player_homes.remove(&home_name).is_some() {
                    target
                        .send_system_message(&TextComponent::text(format!(
                            "Home '{}' has been deleted",
                            home_name
                        )))
                        .await;
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
            .then(argument(ARG_HOME_NAME, SimpleArgConsumer).execute(DelhomeExecutor))
    )
} use pumpkin_util::text::TextComponent;
