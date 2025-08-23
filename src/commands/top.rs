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
    entity::EntityBase,
};
use pumpkin::command::CommandSender::Player;
use pumpkin_util::text::TextComponent;
// TODO: Fix WorldPosition import if needed
// use pumpkin_util::math::position::WorldPosition;

const NAMES: [&str; 1] = ["top"];
const DESCRIPTION: &str = "Teleport to the highest block at your location.";

struct TopExecutor;

#[async_trait]
impl CommandExecutor for TopExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Player(target) = sender {
            // 1. Récupérer la position actuelle du joueur (en flottant)
            let current_pos = target.living_entity.entity.pos.load();
            let x = current_pos.x.floor() as i32;
            let z = current_pos.z.floor() as i32;

            // 2. Récupérer le monde du joueur
            let world = target.living_entity.entity.world.clone();

            // 3. Définir les bornes Y (hauteur du monde)
            // On prend des valeurs classiques, à adapter si besoin
            let min_y = 0;
            let max_y = 320;
            let mut found_y = None;

            // 4. On boucle du haut vers le bas pour trouver le premier bloc solide
            for y in (min_y..=max_y).rev() {
                let pos = pumpkin_util::math::position::BlockPos::new(x, y, z);
                let block_state = world.get_block_state(&pos).await;
                if !block_state.is_air() {
                    found_y = Some(y);
                    break;
                }
            }

            if let Some(y) = found_y {
                // 5. On téléporte le joueur juste au-dessus du bloc trouvé
                let new_pos = pumpkin_util::math::vector3::Vector3::new(current_pos.x, (y + 1) as f64, current_pos.z);
                let yaw = target.living_entity.entity.yaw.load();
                let pitch = target.living_entity.entity.pitch.load();
                let world = target.living_entity.entity.world.clone();
                target.clone().teleport(new_pos, Some(yaw), Some(pitch), world).await;
                target
                    .send_system_message(&TextComponent::text(format!(
                        "Teleported to the top (Y = {})",
                        y + 1
                    )))
                    .await;
            } else {
                // 6. Aucun bloc solide trouvé sous le joueur
                target
                    .send_system_message(&TextComponent::text("No solid block found below you!"))
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
        require(|sender| sender.is_player()).execute(TopExecutor)
    )
}