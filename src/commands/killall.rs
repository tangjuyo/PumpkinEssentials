use async_trait::async_trait;
use pumpkin::{
    command::{
        args::ConsumedArgs,
        dispatcher::CommandError,
        dispatcher::CommandError::InvalidRequirement,
        tree::CommandTree,
        tree::builder::require,
        CommandExecutor, CommandSender,
    },
    server::Server,
};
use pumpkin::command::CommandSender::Player;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["killall"];
const DESCRIPTION: &str = "Kill all entities in the world.";

struct KillallExecutor;

#[async_trait]
impl CommandExecutor for KillallExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            // TODO: Implement entity killing functionality
            // This would require access to the world's entity system
            // For now, we'll show a placeholder message
            
            target
                .send_system_message(&TextComponent::text("Entity killing functionality is not yet implemented."))
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
        require(|sender| sender.is_player()).execute(KillallExecutor)
    )
}