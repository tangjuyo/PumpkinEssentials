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

const NAMES: [&str; 1] = ["suicide"];
const DESCRIPTION: &str = "Commit suicide.";

struct SuicideExecutor;

#[async_trait]
impl CommandExecutor for SuicideExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            // Tuer le joueur en mettant sa vie à 0
            target.set_health(0.0).await;

            Ok(())
        } else {
            Err(InvalidRequirement)
        }
    }
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        require(|sender| sender.is_player()).execute(SuicideExecutor)
    )
}