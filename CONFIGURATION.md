# PumpkinEssentials Configuration Architecture

This document explains the configuration system architecture for the PumpkinEssentials plugin.

## Overview

PumpkinEssentials uses a modular configuration system built on top of the `pumpkin-plugin-config` crate. The system provides:

- **Hierarchical configuration files** with YAML format
- **Type-safe configuration structures** using Serde
- **Automatic default configuration generation**
- **Hot-reload capability**
- **Database configuration support**

## File Structure

```
PumpkinEssentials/
├── config.yml                    # Main configuration file
├── home/                         # Home system configuration
│   ├── config.yml               # Home-specific settings
│   └── data.sqlite              # Home database
└── src/
    └── config.rs                # Configuration management code
```

## Configuration Files

### Main Configuration (`config.yml`)

The main configuration file contains settings for all plugin features:

```yaml
# Plugin Information
plugin:
  name: "PumpkinEssentials"
  version: "1.0.0"
  description: "Essential commands and features for Pumpkin servers"

# General Settings
general:
  language: "en"
  debug: false
  prefix: "&8[&aEssentials&8]&r "

# Command Settings
commands:
  teleport:
    cooldown: 3
    max_distance: 10000
    cross_world: true
    request_timeout: 60
  
  home:
    max_homes: 3
    cooldown: 5
    cross_world: true
  
  spawn:
    cooldown: 10
    enabled: true
  
  warp:
    max_warps: 50
    cooldown: 3
    cross_world: true

# Messages Configuration
messages:
  success:
    teleport: "&aSuccessfully teleported!"
    home_set: "&aHome set successfully!"
    # ... more messages
  
  error:
    teleport_cooldown: "&cPlease wait {cooldown} seconds before teleporting again."
    # ... more error messages
  
  info:
    teleport_request_sent: "&eTeleport request sent to {player}."
    # ... more info messages

# Database Configuration
database:
  type: "sqlite"
  sqlite:
    path: "home/data.sqlite"
  
  mysql:
    host: "localhost"
    port: 3306
    database: "pumpkin_essentials"
    username: "root"
    password: ""

# Permissions
permissions:
  admin:
    - "essentials.admin"
    - "essentials.*"
  
  moderator:
    - "essentials.moderator"
    - "essentials.teleport"
    - "essentials.home"
    - "essentials.spawn"
    - "essentials.warp"
  
  player:
    - "essentials.teleport"
    - "essentials.home"
    - "essentials.spawn"
    - "essentials.warp"
```

### Home Configuration (`home/config.yml`)

The home system has its own detailed configuration:

```yaml
# Home Settings
home:
  max_homes: 3
  cooldown: 5
  cross_world: true
  allow_protected_areas: false
  allow_pvp_zones: false
  allow_nether: true
  allow_end: true
  min_distance_from_spawn: 100
  max_distance_from_spawn: 0
  allow_in_combat: false
  allow_teleport_in_combat: false

# Home Names
home_names:
  default: "home"
  allowed_chars: "^[a-zA-Z0-9_-]+$"
  max_length: 16
  min_length: 1
  reserved:
    - "spawn"
    - "world"
    - "admin"
    - "mod"
    - "staff"

# Home Messages
messages:
  success:
    home_set: "&aHome '{name}' set successfully!"
    home_teleport: "&aWelcome to your home '{name}'!"
    # ... more success messages
  
  error:
    home_limit: "&cYou have reached the maximum number of homes ({max_homes})."
    # ... more error messages
  
  info:
    home_list_header: "&6=== Your Homes ==="
    # ... more info messages

# Home Database
database:
  type: "sqlite"
  sqlite:
    path: "data.sqlite"
  table_name: "homes"
  backup:
    enabled: true
    interval: 24
    keep_count: 7
    directory: "backups"

# Home Permissions
permissions:
  basic:
    - "essentials.home"
    - "essentials.home.set"
    - "essentials.home.teleport"
    - "essentials.home.delete"
    - "essentials.home.list"
  
  admin:
    - "essentials.home.admin"
    - "essentials.home.others"
    - "essentials.home.unlimited"
    - "essentials.home.bypass.cooldown"
    - "essentials.home.bypass.limit"
    - "essentials.home.bypass.restrictions"
```

## Code Architecture

### Configuration Structures

The configuration system uses strongly-typed structures defined in `pumpkin-plugin-config/src/essentials_config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EssentialsConfig {
    pub plugin: PluginInfo,
    pub general: GeneralSettings,
    pub commands: CommandSettings,
    pub messages: MessageSettings,
    pub database: DatabaseSettings,
    pub permissions: PermissionSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeConfig {
    pub home: HomeDetailedSettings,
    pub home_names: HomeNameSettings,
    pub messages: HomeMessageSettings,
    pub database: HomeDatabaseSettings,
    pub permissions: HomePermissionSettings,
}
```

### Configuration Manager

The `EssentialsConfigManager` class handles:

- **Loading and parsing** configuration files
- **Saving default configurations** when files don't exist
- **Hot-reloading** configurations
- **Type-safe access** to configuration values

```rust
pub struct EssentialsConfigManager {
    main_config: Arc<RwLock<EssentialsConfig>>,
    home_config: Arc<RwLock<HomeConfig>>,
    data_folder: PathBuf,
}

impl EssentialsConfigManager {
    pub async fn init(&self) -> Result<()> {
        // Save default configs if they don't exist
        self.save_default_configs().await?;
        
        // Load configurations
        self.load_configs().await?;
        
        Ok(())
    }
    
    pub async fn get_main_config(&self) -> EssentialsConfig {
        self.main_config.read().await.clone()
    }
    
    pub async fn get_home_config(&self) -> HomeConfig {
        self.home_config.read().await.clone()
    }
    
    pub async fn reload(&self) -> Result<()> {
        self.load_configs().await?;
        Ok(())
    }
}
```

## Usage in Commands

Commands can access configuration values through the configuration manager:

```rust
// Example: Using configuration in a command
async fn execute_home_command(player: &Player, config_manager: &EssentialsConfigManager) {
    let config = config_manager.get_home_config().await;
    
    // Check home limit
    if player_home_count >= config.home.max_homes {
        player.send_message(&config.messages.error.get("home_limit")
            .unwrap_or(&"&cHome limit reached!".to_string()));
        return;
    }
    
    // Check cooldown
    if !can_teleport_home(player.uuid, config.home.cooldown).await {
        player.send_message(&config.messages.error.get("home_cooldown")
            .unwrap_or(&"&cPlease wait before teleporting home again!".to_string()));
        return;
    }
    
    // Execute teleport
    player.teleport(home_location).await;
    player.send_message(&config.messages.success.get("home_teleport")
        .unwrap_or(&"&aWelcome home!".to_string()));
}
```

## Database Configuration

The system supports multiple database types:

- **SQLite** (default) - File-based database
- **MySQL** - Remote database server
- **PostgreSQL** - Advanced database server

Database configuration is handled through the `DatabaseSettings` structure:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub r#type: String,
    pub sqlite: SqliteSettings,
    pub mysql: MySqlSettings,
    pub postgresql: PostgreSqlSettings,
}
```

## Message System

The configuration system includes a comprehensive message system with:

- **Success messages** - Positive feedback
- **Error messages** - Error notifications
- **Info messages** - Informational content

Messages support **placeholders** for dynamic content:

```yaml
messages:
  success:
    home_set: "&aHome '{name}' set successfully!"
  error:
    teleport_cooldown: "&cPlease wait {cooldown} seconds before teleporting again."
  info:
    home_count: "&eYou have {current}/{max} homes set."
```

## Permissions

The configuration system defines permission hierarchies:

- **Admin permissions** - Full access to all features
- **Moderator permissions** - Limited administrative access
- **Player permissions** - Basic user access

Permissions are automatically registered when the plugin loads.

## Future Enhancements

Planned improvements to the configuration system:

1. **Embedded resources** - Include default configs in the binary
2. **Configuration validation** - Validate config values at load time
3. **Configuration migration** - Automatic config format updates
4. **Web-based configuration** - Admin interface for configuration
5. **Configuration templates** - Pre-built configs for different server types

## Troubleshooting

### Common Issues

1. **Configuration not loading**: Check file permissions and YAML syntax
2. **Default configs not saving**: Ensure the plugin has write access to the data folder
3. **Database connection errors**: Verify database settings and connectivity

### Debug Mode

Enable debug mode in the configuration to get detailed logging:

```yaml
general:
  debug: true
```

This will log all configuration loading and parsing operations. 