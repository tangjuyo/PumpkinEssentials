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

const NAMES: [&str; 1] = ["kickall"];
const DESCRIPTION: &str = "Kick all players from the server.";

struct KickallExecutor;

#[async_trait]
impl CommandExecutor for KickallExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            // TODO: Implement kick all functionality
            // This would require access to the server's player list and kick functionality
            // For now, we'll show a placeholder message
            
            let player_count = server.get_player_count().await;
            
            target
                .send_system_message(&TextComponent::text(format!(
                    "Kick all functionality is not yet implemented. (Would kick {} players)",
                    player_count
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
        require(|sender| sender.is_player()).execute(KickallExecutor)
    )
}