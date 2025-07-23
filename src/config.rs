//! Configuration management for PumpkinEssentials
//!
//! This module uses the automatic configuration system from pumpkin-plugin-config.

use anyhow::Result;
use pumpkin_plugin_config::AutoConfigManager;
use std::path::PathBuf;
use pumpkin_plugin_config::ConfigurationSection;

mod essentials_config;
pub use essentials_config::{EssentialsConfig, HomeConfig};

/// Configuration manager for PumpkinEssentials
/// 
/// This is a simple wrapper around AutoConfigManager that provides
/// type-safe access to Essentials-specific configurations.
pub struct EssentialsConfigManager {
    /// The underlying auto configuration manager
    auto_manager: AutoConfigManager,
}

impl EssentialsConfigManager {
    /// Create a new configuration manager
    pub fn new(data_folder: PathBuf) -> Self {
        // Use the auto_config_manager_with_resources macro to include specific resources
        let auto_manager = pumpkin_plugin_config::auto_config_manager_with_resources!("PumpkinEssentials", data_folder, [
            "config.yml",
            "home/home_config.yml",
            "home/data.yml",
            "warp_config.yml"
        ]);
        
        Self { auto_manager }
    }
    
    /// Initialize the configuration system
    pub async fn init(&self) -> Result<()> {
        self.auto_manager.init().await?;
        Ok(())
    }
    
    /// Get the main configuration
    pub async fn get_main_config(&self) -> Result<EssentialsConfig> {
        self.auto_manager.get_main_config().await
    }
    
    /// Get the home configuration
    pub async fn get_home_config(&self) -> Result<HomeConfig> {
        self.auto_manager.get_config_file("home_config.yml").await
    }
    
    /// Get the warp configuration
    pub async fn get_warp_config(&self) -> Result<serde_yaml::Value> {
        self.auto_manager.get_config_file("warp_config.yml").await
    }
    
    /// Reload all configurations
    pub async fn reload(&self) -> Result<()> {
        self.auto_manager.reload().await
    }
    
    /// Test function to verify embedded resources are working
    pub fn test_embedded_resources() {
        log::info!("[Essentials] Testing embedded resources...");
        
        // The auto_config_manager macro will handle this automatically
        log::info!("[Essentials] Embedded resources test completed");
    }

    /// Get warp configuration as a configuration section
    /// 
    /// This demonstrates how to use the new ConfigurationSection functionality
    /// similar to Bukkit's config.getConfigurationSection().
    /// 
    /// # Example
    /// ```rust
    /// if let Some(warp_section) = config_manager.get_warp_configuration_section() {
    ///     let max_warps = warp_section.get_int("max_warps", 10);
    ///     let cooldown = warp_section.get_int("cooldown", 5);
    ///     let cross_world = warp_section.get_bool("cross_world", true);
    ///     
    ///     // Access nested values
    ///     if let Some(permissions_section) = warp_section.get_configuration_section("permissions") {
    ///         let create_permission = permissions_section.get_string("create", "essentials.warp.create");
    ///         let use_permission = permissions_section.get_string("use", "essentials.warp.use");
    ///     }
    /// }
    /// ```
    pub async fn get_warp_configuration_section(&self) -> Option<ConfigurationSection> {
        self.auto_manager.get_file_configuration_section("warp_config.yml", "").await
    }

    /// Get home configuration as a configuration section
    /// 
    /// This demonstrates how to use the new ConfigurationSection functionality
    /// for accessing nested configuration values.
    /// 
    /// # Example
    /// ```rust
    /// if let Some(home_section) = config_manager.get_home_configuration_section() {
    ///     let max_homes = home_section.get_int("home.max_homes", 3);
    ///     let cooldown = home_section.get_int("home.cooldown", 5);
    ///     let cross_world = home_section.get_bool("home.cross_world", true);
    ///     
    ///     // Access nested settings
    ///     if let Some(settings_section) = home_section.get_configuration_section("home") {
    ///         let allow_protected = settings_section.get_bool("allow_protected_areas", false);
    ///         let allow_pvp = settings_section.get_bool("allow_pvp_zones", true);
    ///         let min_distance = settings_section.get_double("min_distance_from_spawn", 100.0);
    ///     }
    /// }
    /// ```
    pub async fn get_home_configuration_section(&self) -> Option<ConfigurationSection> {
        self.auto_manager.get_file_configuration_section("home/home_config.yml", "").await
    }

    /// Get main configuration as a configuration section
    /// 
    /// This demonstrates how to use the new ConfigurationSection functionality
    /// for the main config file.
    /// 
    /// # Example
    /// ```rust
    /// if let Some(main_section) = config_manager.get_main_configuration_section() {
    ///     let plugin_name = main_section.get_string("plugin.name", "PumpkinEssentials");
    ///     let version = main_section.get_string("plugin.version", "1.0.0");
    ///     let debug = main_section.get_bool("general.debug", false);
    ///     let prefix = main_section.get_string("general.prefix", "&8[&aEssentials&8]&r ");
    ///     
    ///     // Access command-specific settings
    ///     if let Some(teleport_section) = main_section.get_configuration_section("commands.teleport") {
    ///         let cooldown = teleport_section.get_int("cooldown", 3);
    ///         let max_distance = teleport_section.get_double("max_distance", 10000.0);
    ///         let cross_world = teleport_section.get_bool("cross_world", true);
    ///     }
    /// }
    /// ```
    pub async fn get_main_configuration_section(&self) -> Option<ConfigurationSection> {
        self.auto_manager.get_configuration_section("").await
    }

    /// Get teleport configuration values
    /// 
    /// # Returns
    /// * `(i64, f64, bool, i64)` - (cooldown, max_distance, cross_world, request_timeout)
    /// 
    /// # Example
    /// ```rust
    /// let (cooldown, max_distance, cross_world, timeout) = config_manager.get_teleport_config().await;
    /// ```
    pub async fn get_teleport_config(&self) -> (i64, f64, bool, i64) {
        let cooldown = self.auto_manager.get_int("commands.teleport.cooldown", 3).await;
        let max_distance = self.auto_manager.get_double("commands.teleport.max_distance", 10000.0).await;
        let cross_world = self.auto_manager.get_bool("commands.teleport.cross_world", true).await;
        let request_timeout = self.auto_manager.get_int("commands.teleport.request_timeout", 60).await;
        
        (cooldown, max_distance, cross_world, request_timeout)
    }

    /// Get home configuration values from main config
    /// 
    /// # Returns
    /// * `(i64, i64, bool)` - (max_homes, cooldown, cross_world)
    /// 
    /// # Example
    /// ```rust
    /// let (max_homes, cooldown, cross_world) = config_manager.get_home_config_main().await;
    /// ```
    pub async fn get_home_config_main(&self) -> (i64, i64, bool) {
        let max_homes = self.auto_manager.get_int("commands.home.max_homes", 3).await;
        let cooldown = self.auto_manager.get_int("commands.home.cooldown", 5).await;
        let cross_world = self.auto_manager.get_bool("commands.home.cross_world", true).await;
        
        (max_homes, cooldown, cross_world)
    }

    /// Get detailed home configuration from home config file
    /// 
    /// # Returns
    /// * `Option<(i64, i64, bool, bool, bool, f64, f64, bool, bool)>` - Detailed home settings
    /// 
    /// # Example
    /// ```rust
    /// if let Some((max_homes, cooldown, cross_world, allow_protected, allow_pvp, min_distance, max_distance, allow_combat, allow_teleport_combat)) = config_manager.get_home_config_detailed().await {
    ///     // Use detailed home settings
    /// }
    /// ```
    pub async fn get_home_config_detailed(&self) -> Option<(i64, i64, bool, bool, bool, f64, f64, bool, bool)> {
        if let Some(home_section) = self.get_home_configuration_section().await {
            if let Some(settings_section) = home_section.get_configuration_section("home") {
                let max_homes = settings_section.get_int("max_homes", 3);
                let cooldown = settings_section.get_int("cooldown", 5);
                let cross_world = settings_section.get_bool("cross_world", true);
                let allow_protected = settings_section.get_bool("allow_protected_areas", false);
                let allow_pvp = settings_section.get_bool("allow_pvp_zones", true);
                let min_distance = settings_section.get_double("min_distance_from_spawn", 100.0);
                let max_distance = settings_section.get_double("max_distance_from_spawn", 10000.0);
                let allow_combat = settings_section.get_bool("allow_in_combat", false);
                let allow_teleport_combat = settings_section.get_bool("allow_teleport_in_combat", false);
                
                Some((max_homes, cooldown, cross_world, allow_protected, allow_pvp, min_distance, max_distance, allow_combat, allow_teleport_combat))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get spawn configuration values
    /// 
    /// # Returns
    /// * `(i64, bool)` - (cooldown, enabled)
    /// 
    /// # Example
    /// ```rust
    /// let (cooldown, enabled) = config_manager.get_spawn_config().await;
    /// ```
    pub async fn get_spawn_config(&self) -> (i64, bool) {
        let cooldown = self.auto_manager.get_int("commands.spawn.cooldown", 10).await;
        let enabled = self.auto_manager.get_bool("commands.spawn.enabled", true).await;
        
        (cooldown, enabled)
    }

    /// Get warp configuration values
    /// 
    /// # Returns
    /// * `(i64, i64, bool)` - (max_warps, cooldown, cross_world)
    /// 
    /// # Example
    /// ```rust
    /// let (max_warps, cooldown, cross_world) = config_manager.get_warp_config_main().await;
    /// ```
    pub async fn get_warp_config_main(&self) -> (i64, i64, bool) {
        let max_warps = self.auto_manager.get_int("commands.warp.max_warps", 50).await;
        let cooldown = self.auto_manager.get_int("commands.warp.cooldown", 3).await;
        let cross_world = self.auto_manager.get_bool("commands.warp.cross_world", true).await;
        
        (max_warps, cooldown, cross_world)
    }

    /// Get detailed warp configuration from warp config file
    /// 
    /// # Returns
    /// * `Option<(i64, i64, bool)>` - Detailed warp settings
    /// 
    /// # Example
    /// ```rust
    /// if let Some((max_warps, cooldown, cross_world)) = config_manager.get_warp_config_detailed().await {
    ///     // Use detailed warp settings
    /// }
    /// ```
    pub async fn get_warp_config_detailed(&self) -> Option<(i64, i64, bool)> {
        if let Some(warp_section) = self.get_warp_configuration_section().await {
            let max_warps = warp_section.get_int("max_warps", 50);
            let cooldown = warp_section.get_int("cooldown", 3);
            let cross_world = warp_section.get_bool("cross_world", true);
            
            Some((max_warps, cooldown, cross_world))
        } else {
            None
        }
    }

    /// Get plugin information
    /// 
    /// # Returns
    /// * `(String, String, String)` - (name, version, description)
    /// 
    /// # Example
    /// ```rust
    /// let (name, version, description) = config_manager.get_plugin_info().await;
    /// ```
    pub async fn get_plugin_info(&self) -> (String, String, String) {
        let name = self.auto_manager.get_string("plugin.name", "PumpkinEssentials").await;
        let version = self.auto_manager.get_string("plugin.version", "1.0.0").await;
        let description = self.auto_manager.get_string("plugin.description", "Essential commands and features for Pumpkin servers").await;
        
        (name, version, description)
    }

    /// Get general settings
    /// 
    /// # Returns
    /// * `(String, bool, String)` - (language, debug, prefix)
    /// 
    /// # Example
    /// ```rust
    /// let (language, debug, prefix) = config_manager.get_general_settings().await;
    /// ```
    pub async fn get_general_settings(&self) -> (String, bool, String) {
        let language = self.auto_manager.get_string("general.language", "en").await;
        let debug = self.auto_manager.get_bool("general.debug", false).await;
        let prefix = self.auto_manager.get_string("general.prefix", "&8[&aEssentials&8]&r ").await;
        
        (language, debug, prefix)
    }

    /// Get a message from the configuration
    /// 
    /// # Arguments
    /// * `category` - Message category (success, error, info)
    /// * `key` - Message key
    /// * `default` - Default message if not found
    /// 
    /// # Returns
    /// * `String` - The message
    /// 
    /// # Example
    /// ```rust
    /// let message = config_manager.get_message("success", "home_teleport", "&aWelcome home!").await;
    /// ```
    pub async fn get_message(&self, category: &str, key: &str, default: &str) -> String {
        let path = format!("messages.{}.{}", category, key);
        self.auto_manager.get_string(&path, default).await
    }

    /// Get a success message
    pub async fn get_success_message(&self, key: &str, default: &str) -> String {
        self.get_message("success", key, default).await
    }

    /// Get an error message
    pub async fn get_error_message(&self, key: &str, default: &str) -> String {
        self.get_message("error", key, default).await
    }

    /// Get an info message
    pub async fn get_info_message(&self, key: &str, default: &str) -> String {
        self.get_message("info", key, default).await
    }

    /// Get database configuration
    /// 
    /// # Returns
    /// * `(String, String)` - (database_type, sqlite_path)
    /// 
    /// # Example
    /// ```rust
    /// let (db_type, sqlite_path) = config_manager.get_database_config().await;
    /// ```
    pub async fn get_database_config(&self) -> (String, String) {
        let db_type = self.auto_manager.get_string("database.type", "sqlite").await;
        let sqlite_path = self.auto_manager.get_string("database.sqlite.path", "home/data.yml").await;
        
        (db_type, sqlite_path)
    }

    /// Get permissions list
    /// 
    /// # Arguments
    /// * `role` - Role (admin, moderator, player)
    /// 
    /// # Returns
    /// * `Option<Vec<String>>` - List of permissions
    /// 
    /// # Example
    /// ```rust
    /// if let Some(admin_perms) = config_manager.get_permissions("admin").await {
    ///     for perm in admin_perms {
    ///         println!("Admin permission: {}", perm);
    ///     }
    /// }
    /// ```
    pub async fn get_permissions(&self, role: &str) -> Option<Vec<String>> {
        let path = format!("permissions.{}", role);
        self.auto_manager.get_string_list(&path).await
    }

    /// Check if debug mode is enabled
    pub async fn is_debug_enabled(&self) -> bool {
        self.auto_manager.get_bool("general.debug", false).await
    }

    /// Get the plugin prefix
    pub async fn get_prefix(&self) -> String {
        self.auto_manager.get_string("general.prefix", "&8[&aEssentials&8]&r ").await
    }
} 