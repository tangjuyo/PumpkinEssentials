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

use super::home_common::{ARG_HOME_NAME, PLAYER_HOMES};

const NAMES: [&str; 1] = ["sethome"];
const DESCRIPTION: &str = "Set your home at your current location.";

// /sethome command
struct SethomeExecutor;

#[async_trait]
impl CommandExecutor for SethomeExecutor {
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

            // Get current position
            let position = target.living_entity.entity.pos.load();
            let yaw = target.living_entity.entity.yaw.load();
            let pitch = target.living_entity.entity.pitch.load();

            // Add the home to the player's homes
            homes
                .entry(target.gameprofile.id)
                .or_insert_with(|| std::collections::HashMap::new())
                .insert(home_name.clone(), (position, yaw, pitch));

            target
                .send_system_message(&TextComponent::text(format!(
                    "Home '{}' set at your current location",
                    home_name
                )))
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
        require(|sender| sender.is_player())
            .execute(SethomeExecutor)
            .then(argument(ARG_HOME_NAME, SimpleArgConsumer).execute(SethomeExecutor)),
    )
}
use pumpkin_util::text::TextComponent;
