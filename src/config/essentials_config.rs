//! Essentials-specific configuration structures
//!
//! This module contains the configuration structures specific to PumpkinEssentials.
//! These are kept separate from the generic pumpkin-plugin-config crate.

use serde::{Deserialize, Serialize};

/// Main configuration for PumpkinEssentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EssentialsConfig {
    pub plugin: PluginConfig,
    pub general: GeneralConfig,
    pub commands: CommandsConfig,
    pub database: DatabaseConfig,
}

impl Default for EssentialsConfig {
    fn default() -> Self {
        Self {
            plugin: PluginConfig::default(),
            general: GeneralConfig::default(),
            commands: CommandsConfig::default(),
            database: DatabaseConfig::default(),
        }
    }
}

/// Plugin information configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub version: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            name: "PumpkinEssentials".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

/// General plugin settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub language: String,
    pub debug: bool,
    pub prefix: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            debug: false,
            prefix: "&8[&aEssentials&8]&r ".to_string(),
        }
    }
}

/// Command-specific configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandsConfig {
    pub teleport: TeleportConfig,
    pub home: HomeCommandConfig,
    pub spawn: SpawnConfig,
    pub warp: WarpConfig,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            teleport: TeleportConfig::default(),
            home: HomeCommandConfig::default(),
            spawn: SpawnConfig::default(),
            warp: WarpConfig::default(),
        }
    }
}

/// Teleport command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeleportConfig {
    pub cooldown: u64,
    pub max_distance: f64,
    pub cross_world: bool,
    pub request_timeout: u64,
}

impl Default for TeleportConfig {
    fn default() -> Self {
        Self {
            cooldown: 3,
            max_distance: 10000.0,
            cross_world: true,
            request_timeout: 60,
        }
    }
}

/// Home command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeCommandConfig {
    pub max_homes: u32,
    pub cooldown: u64,
    pub cross_world: bool,
}

impl Default for HomeCommandConfig {
    fn default() -> Self {
        Self {
            max_homes: 3,
            cooldown: 5,
            cross_world: true,
        }
    }
}

/// Spawn command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnConfig {
    pub cooldown: u64,
    pub enabled: bool,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            cooldown: 3,
            enabled: true,
        }
    }
}

/// Warp command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpConfig {
    pub max_warps: u32,
    pub cooldown: u64,
    pub cross_world: bool,
}

impl Default for WarpConfig {
    fn default() -> Self {
        Self {
            max_warps: 10,
            cooldown: 5,
            cross_world: true,
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(rename = "type")]
    pub r#type: String,
    pub sqlite: SqliteConfig,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            r#type: "sqlite".to_string(),
            sqlite: SqliteConfig::default(),
        }
    }
}

/// SQLite-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig {
    pub path: String,
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            path: "data/essentials.sqlite".to_string(),
        }
    }
}

/// Home system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeConfig {
    pub home: HomeSettings,
    pub home_names: HomeNamesConfig,
    pub database: HomeDatabaseConfig,
}

impl Default for HomeConfig {
    fn default() -> Self {
        Self {
            home: HomeSettings::default(),
            home_names: HomeNamesConfig::default(),
            database: HomeDatabaseConfig::default(),
        }
    }
}

/// Home settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeSettings {
    pub max_homes: u32,
    pub cooldown: u64,
    pub cross_world: bool,
    pub allow_protected_areas: bool,
    pub allow_pvp_zones: bool,
    pub allow_nether: bool,
    pub allow_end: bool,
    pub min_distance_from_spawn: f64,
    pub max_distance_from_spawn: f64,
    pub allow_in_combat: bool,
    pub allow_teleport_in_combat: bool,
}

impl Default for HomeSettings {
    fn default() -> Self {
        Self {
            max_homes: 3,
            cooldown: 5,
            cross_world: true,
            allow_protected_areas: false,
            allow_pvp_zones: true,
            allow_nether: true,
            allow_end: true,
            min_distance_from_spawn: 100.0,
            max_distance_from_spawn: 10000.0,
            allow_in_combat: false,
            allow_teleport_in_combat: false,
        }
    }
}

/// Home names configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeNamesConfig {
    pub default: String,
    pub allowed_chars: String,
    pub max_length: u32,
    pub min_length: u32,
}

impl Default for HomeNamesConfig {
    fn default() -> Self {
        Self {
            default: "home".to_string(),
            allowed_chars: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-".to_string(),
            max_length: 16,
            min_length: 1,
        }
    }
}

/// Home database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeDatabaseConfig {
    #[serde(rename = "type")]
    pub r#type: String,
    pub sqlite: HomeSqliteConfig,
    pub table_name: String,
}

impl Default for HomeDatabaseConfig {
    fn default() -> Self {
        Self {
            r#type: "sqlite".to_string(),
            sqlite: HomeSqliteConfig::default(),
            table_name: "homes".to_string(),
        }
    }
}

/// Home SQLite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeSqliteConfig {
    pub path: String,
}

impl Default for HomeSqliteConfig {
    fn default() -> Self {
        Self {
            path: "data/homes.sqlite".to_string(),
        }
    }
} 