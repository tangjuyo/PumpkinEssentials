use std::sync::Arc;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use pumpkin::{
    plugin::{
        Context, EventPriority,
        player::player_teleport::PlayerTeleportEvent,
    },
    server::Server,
};
use pumpkin_api_macros::{plugin_impl, plugin_method};
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault},
};
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use uuid::Uuid;
use pumpkin::PLUGIN_MANAGER;
use pumpkin::plugin::api::events::player::player_join::PlayerJoinEvent;

mod commands;
mod config;
mod models;
mod managers;

const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

// Cooldown system to prevent spam teleportation
static TELEPORT_COOLDOWNS: Lazy<Arc<Mutex<HashMap<Uuid, Instant>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// Fly state tracking system
static FLY_STATES: Lazy<Arc<Mutex<HashMap<Uuid, bool>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

const TELEPORT_COOLDOWN_DURATION: Duration = Duration::from_millis(500); // 500ms cooldown

// Helper function to check if a player can teleport (not on cooldown)
async fn can_teleport(player_uuid: Uuid) -> bool {
    let mut cooldowns = TELEPORT_COOLDOWNS.lock().await;
    let now = Instant::now();
    
    if let Some(last_teleport) = cooldowns.get(&player_uuid) {
        if now.duration_since(*last_teleport) < TELEPORT_COOLDOWN_DURATION {
            return false;
        }
    }
    
    cooldowns.insert(player_uuid, now);
    true

}

// Helper function to get fly state for a player
pub async fn get_fly_state(player_uuid: Uuid) -> bool {
    let fly_states = FLY_STATES.lock().await;
    fly_states.get(&player_uuid).copied().unwrap_or(false)
}

// Helper function to set fly state for a player
pub async fn set_fly_state(player_uuid: Uuid, state: bool) {
    let mut fly_states = FLY_STATES.lock().await;
    fly_states.insert(player_uuid, state);
}

async fn register_commands(context: &Context) -> Result<(), String> {
    // Register permissions for all commands
    let commands_list = [
        ("home", "Teleport to your home"),
        ("sethome", "Set a home location"),
        ("delhome", "Delete a home location"),
        ("renamehome", "Rename a home location"),
        ("homes", "List all your homes"),
        ("back", "Teleport to your previous location"),
        ("gmc", "Change to creative mode"),
        ("gms", "Change to survival mode"),
        ("gma", "Change to adventure mode"),
        ("gmsp", "Change to spectator mode"),
        ("top", "Teleport to the highest block"),
        ("tpa", "Request to teleport to another player"),
        ("tpaccept", "Accept a teleport request"),
        ("tpdeny", "Deny a teleport request"),
        ("tpahere", "Request another player to teleport to you"),
        ("tpall", "Teleport all players to you"),
        ("heal", "Heal yourself or another player"),
        ("feed", "Feed yourself or another player"),
        ("fly", "Toggle flight mode"),
        ("god", "Toggle god mode"),
        ("ping", "Check ping"),
        ("repair", "Repair the item in your hand"),
        ("suicide", "Commit suicide"),
        ("killall", "Kill all entities"),
        ("enderchest", "Open an enderchest"),
        ("ignore", "Ignore a player"),
        ("kickall", "Kick all players"),
        ("sudo", "Execute a command as another player"),
        ("speed", "Set walk or fly speed"),
        ("spawn", "Teleport to the server spawn point"),
        ("setspawn", "Set the server spawn point to your current location"),
        ("near", "Show nearby players within a specified radius"),
        ("createkit", "Create a kit from your current inventory"),
        ("kit", "Use a kit or give it to another player"),
        ("resetkit", "Reset the cooldown of a kit for a player"),
    ];

    for (cmd, description) in commands_list.iter() {
        let permission = Permission::new(
            &format!("{PLUGIN_NAME}:command.{}", cmd),
            description,
            PermissionDefault::Op(PermissionLvl::One), // Allow all players by default
        );
        context.register_permission(permission).await?;
    }

    // Register all our commands
    context.register_command(
        commands::home::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.home")
    ).await;
    
    context.register_command(
        commands::sethome::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.sethome")
    ).await;
    
    context.register_command(
        commands::delhome::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.delhome")
    ).await;
    
    context.register_command(
        commands::renamehome::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.renamehome")
    ).await;
    
    context.register_command(
        commands::homes::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.homes")
    ).await;
    
    context.register_command(
        commands::back::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.back")
    ).await;
    
    context.register_command(
        commands::gmc::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.gmc")
    ).await;
    
    context.register_command(
        commands::gms::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.gms")
    ).await;
    
    context.register_command(
        commands::gma::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.gma")
    ).await;
    
    context.register_command(
        commands::gmsp::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.gmsp")
    ).await;
    
    context.register_command(
        commands::top::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.top")
    ).await;
    
    context.register_command(
        commands::tpa::init_tpa_command_tree(),
        &format!("{PLUGIN_NAME}:command.tpa")
    ).await;
    
    context.register_command(
        commands::tpa::init_tpaccept_command_tree(),
        &format!("{PLUGIN_NAME}:command.tpaccept")
    ).await;
    
    context.register_command(
        commands::tpa::init_tpdeny_command_tree(),
        &format!("{PLUGIN_NAME}:command.tpdeny")
    ).await;
    
    context.register_command(
        commands::tpahere::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.tpahere")
    ).await;
    
    context.register_command(
        commands::tpall::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.tpall")
    ).await;

    context.register_command(
        commands::heal::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.heal")
    ).await;

    context.register_command(
        commands::feed::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.feed")
    ).await;

    context.register_command(
        commands::fly::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.fly")
    ).await;

    context.register_command(
        commands::god::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.god")
    ).await;

    context.register_command(
        commands::ping::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.ping")
    ).await;

    context.register_command(
        commands::repair::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.repair")
    ).await;

    context.register_command(
        commands::suicide::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.suicide")
    ).await;

    context.register_command(
        commands::killall::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.killall")
    ).await;

    context.register_command(
        commands::enderchest::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.enderchest")
    ).await;

    context.register_command(
        commands::ignore::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.ignore")
    ).await;

    context.register_command(
        commands::kickall::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.kickall")
    ).await;

    context.register_command(
        commands::sudo::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.sudo")
    ).await;

    context.register_command(
        commands::speed::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.speed")
    ).await;

    context.register_command(
        commands::spawn::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.spawn")
    ).await;
    
    context.register_command(
        commands::setspawn::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.setspawn")
    ).await;
    
    context.register_command(
        commands::near::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.near")
    ).await;
    
    context.register_command(
        commands::createkit::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.createkit")
    ).await;
    
    context.register_command(
        commands::kit::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.kit")
    ).await;
    
    context.register_command(
        commands::resetkit::init_command_tree(),
        &format!("{PLUGIN_NAME}:command.resetkit")
    ).await;

    Ok(())
}

async fn register_events(context: &Context) {
    log::info!("[Essentials] Appel de register_events : enregistrement du handler BackLocationHandler");
    PLUGIN_MANAGER.read().await.register::<PlayerTeleportEvent, commands::back::BackLocationHandler>(
        Arc::new(commands::back::BackLocationHandler),
        EventPriority::Normal,
        false
    ).await;

    // TEST: Enregistrement d'un handler pour PlayerJoinEvent
    PLUGIN_MANAGER.read().await.register::<PlayerJoinEvent, TestJoinHandler>(
        Arc::new(TestJoinHandler),
        EventPriority::Normal,
        false
    ).await;
}

// Handler de test pour PlayerJoinEvent
struct TestJoinHandler;

#[async_trait::async_trait]
impl pumpkin::plugin::EventHandler<PlayerJoinEvent> for TestJoinHandler {
    async fn handle_blocking(&self, _server: &Arc<Server>, event: &mut PlayerJoinEvent) {
        log::info!("[Essentials] Handler TestJoinHandler appelé pour {}", event.player.gameprofile.name);
    }
}#[plugin_method]
async fn on_load(&mut self, context: &Context) -> Result<(), String> {
    pumpkin::init_log!();

    // Test embedded resources
    config::EssentialsConfigManager::test_embedded_resources();
    
    // Initialize configuration system
    let data_folder = context.get_data_folder();
    let config_manager = config::EssentialsConfigManager::new(data_folder.clone());
    
    if let Err(e) = config_manager.init().await {
        log::error!("[Essentials] Failed to initialize configuration: {}", e);
        return Err(format!("Configuration initialization failed: {}", e));
    }
    
    // Test the new ConfigurationSection functionality
    log::info!("[Essentials] Testing new ConfigurationSection features...");
    
    // Store the config manager in the context for later use
    // TODO: Implement proper storage mechanism
    
    // Initialize home manager with database
    let data_folder_clone = data_folder.clone();
    if let Err(e) = managers::home_manager::init_home_manager(data_folder_clone).await {
        log::error!("[Essentials] Failed to initialize home manager: {}", e);
        return Err(format!("Home manager initialization failed: {}", e));
    }
    
    // Initialize spawn manager
    if let Err(e) = managers::spawn_manager::init_spawn_manager(data_folder.clone()).await {
        log::error!("[Essentials] Failed to initialize spawn manager: {}", e);
        return Err(format!("Spawn manager initialization failed: {}", e));
    }
    
    // Initialize kit manager
    match managers::kit_manager::KitManager::new(data_folder.clone()).await {
        Ok(kit_manager) => {
            managers::kit_manager::set_kit_manager(Arc::new(kit_manager)).await;
            log::info!("[Essentials] Kit manager initialized successfully");
        }
        Err(e) => {
            log::error!("[Essentials] Failed to initialize kit manager: {}", e);
            return Err(format!("Kit manager initialization failed: {}", e));
        }
    }
    
    log::info!("[Essentials] Home manager initialized successfully");

    register_commands(context).await?;
    register_events(context).await;

    log::info!("PumpkinEssentials Plugin has been loaded successfully.");
    Ok(())
}

#[plugin_impl]
pub struct Plugin {}

impl Plugin {
    pub fn new() -> Self {
        Plugin {}
    }
}

impl Default for Plugin {
    fn default() -> Self {
        Self::new()
    }
}

