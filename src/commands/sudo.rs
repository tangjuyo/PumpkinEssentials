use async_trait::async_trait;
use pumpkin::{
    command::{
        args::{Arg, ConsumedArgs, players::PlayersArgumentConsumer, message::MsgArgConsumer},
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

const NAMES: [&str; 1] = ["sudo"];
const DESCRIPTION: &str = "Execute a command as another player.";
const ARG_TARGET: &str = "target";
const ARG_COMMAND: &str = "command";

struct SudoExecutor;

#[async_trait]
impl CommandExecutor for SudoExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(executor) = sender {
            let target_player = if let Some(Arg::Players(players)) = args.get(ARG_TARGET) {
                if players.len() == 1 {
                    players[0].clone()
                } else {
                    return Err(InvalidConsumption(Some("Expected exactly one player".to_string())));
                }
            } else {
                return Err(InvalidConsumption(Some("Target player is required".to_string())));
            };

            let command = if let Some(Arg::Msg(cmd)) = args.get(ARG_COMMAND) {
                cmd.clone()
            } else {
                return Err(InvalidConsumption(Some("Command is required".to_string())));
            };

            // Sauvegarder l'ancien niveau de permission
            let old_lvl = target_player.permission_lvl.load();
            let new_lvl = executor.permission_lvl.load();
            target_player.permission_lvl.store(new_lvl);

            // Exécuter la commande comme le joueur cible
            let dispatcher = server.command_dispatcher.read().await;
            let mut target_sender = CommandSender::Player(target_player.clone());
            dispatcher.handle_command(&mut target_sender, server, &command).await;

            // Restaurer l'ancien niveau de permission
            target_player.permission_lvl.store(old_lvl);

            let player_name = &target_player.gameprofile.name;
            executor
                .send_system_message(&TextComponent::text(format!(
                    "Tried to execute '{}' as {} (see their chat for result)",
                    command, player_name
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
            .then(argument(ARG_TARGET, PlayersArgumentConsumer)
                .then(argument(ARG_COMMAND, MsgArgConsumer).execute(SudoExecutor)))
    )
}