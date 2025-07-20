# 🎯 ConfigurationSection Usage in PumpkinEssentials

This document explains how to use the new `ConfigurationSection` functionality in PumpkinEssentials, demonstrating practical examples of the Bukkit-like configuration access patterns.

## 🚀 **Overview**

PumpkinEssentials now uses the new `ConfigurationSection` system from `pumpkin-plugin-config`, providing:

- ✅ **Bukkit-like syntax** - Familiar API for Java developers
- ✅ **Type-safe access** - Compile-time checking with Rust
- ✅ **Dot notation paths** - Easy navigation in nested configurations
- ✅ **Default values** - Safe access to missing configuration values
- ✅ **Multiple file support** - Access different configuration files easily

## 📝 **Basic Usage Examples**

### **1. Direct Configuration Access**

```rust
// Get configuration values directly (like Bukkit's config.getInt(), config.getString(), etc.)
let config_manager = EssentialsConfigManager::new(data_folder);

// Get plugin information
let (name, version, description) = config_manager.get_plugin_info().await;
println!("Plugin: {} v{} - {}", name, version, description);

// Get general settings
let (language, debug, prefix) = config_manager.get_general_settings().await;
println!("Language: {}, Debug: {}, Prefix: '{}'", language, debug, prefix);

// Get command-specific configurations
let (cooldown, max_distance, cross_world, timeout) = config_manager.get_teleport_config().await;
println!("Teleport - Cooldown: {}s, Max distance: {}, Cross world: {}", 
    cooldown, max_distance, cross_world);
```

### **2. Configuration Sections (Bukkit-style)**

```rust
// Get a configuration section (like Bukkit's config.getConfigurationSection())
if let Some(main_section) = config_manager.get_main_configuration_section().await {
    // Access nested sections
    if let Some(teleport_section) = main_section.get_configuration_section("commands.teleport") {
        let cooldown = teleport_section.get_int("cooldown", 3);
        let max_distance = teleport_section.get_double("max_distance", 10000.0);
        let cross_world = teleport_section.get_bool("cross_world", true);
        
        println!("Teleport section - Cooldown: {}s, Max distance: {}, Cross world: {}", 
            cooldown, max_distance, cross_world);
    }
}
```

### **3. File-Specific Configuration Sections**

```rust
// Get configuration from specific files
if let Some(home_section) = config_manager.get_home_configuration_section().await {
    if let Some(settings_section) = home_section.get_configuration_section("home") {
        let max_homes = settings_section.get_int("max_homes", 3);
        let allow_protected = settings_section.get_bool("allow_protected_areas", false);
        let min_distance = settings_section.get_double("min_distance_from_spawn", 100.0);
        
        println!("Home settings - Max homes: {}, Allow protected: {}, Min distance: {}", 
            max_homes, allow_protected, min_distance);
    }
}
```

## 🏗️ **Practical Examples in Commands**

### **Example 1: Home Command with Configuration**

```rust
// In src/commands/home.rs
async fn execute_home_command(player: &Player, config_manager: &EssentialsConfigManager) {
    // Get home configuration from main config
    let (max_homes, cooldown, cross_world) = config_manager.get_home_config_main().await;
    
    // Get detailed home configuration if available
    let detailed_config = config_manager.get_home_config_detailed().await;
    
    // Get messages from configuration
    let success_message = config_manager.get_success_message("home_teleport", "&aWelcome home!").await;
    let error_message = config_manager.get_error_message("home_not_found", "&cHome not found!").await;
    
    // Check if debug mode is enabled
    let debug_enabled = config_manager.is_debug_enabled().await;
    
    if debug_enabled {
        log::info!("[Home Command] Player {} attempting to teleport to home", player.name);
    }
    
    // Use configuration values in logic
    if player_home_count >= max_homes {
        let limit_message = config_manager.get_error_message("home_limit", 
            "&cYou have reached the maximum number of homes").await;
        player.send_message(&limit_message);
        return;
    }
    
    // Check detailed configuration if available
    if let Some((_, _, _, _, _, min_distance, max_distance, allow_combat, _)) = detailed_config {
        // Example: Check distance from spawn
        let spawn_distance = get_distance_from_spawn(player.position);
        
        if spawn_distance < min_distance {
            let distance_message = config_manager.get_error_message("too_close_to_spawn", 
                "&cYou cannot set a home this close to spawn").await;
            player.send_message(&distance_message);
            return;
        }
    }
    
    // Execute teleport
    player.teleport(home_location).await;
    
    // Use configured success message
    let formatted_message = success_message.replace("{home_name}", &home_name);
    player.send_message(&formatted_message);
}
```

### **Example 2: SetHome Command with Configuration**

```rust
// In src/commands/sethome.rs
async fn execute_sethome_command(player: &Player, config_manager: &EssentialsConfigManager) {
    // Get home configuration
    let (max_homes, cooldown, cross_world) = config_manager.get_home_config_main().await;
    let detailed_config = config_manager.get_home_config_detailed().await;
    
    // Get messages
    let success_message = config_manager.get_success_message("home_set", "&aHome set successfully!").await;
    let limit_message = config_manager.get_error_message("home_limit", 
        "&cYou have reached the maximum number of homes").await;
    
    // Check home limit
    if player_home_count >= max_homes {
        let formatted_message = limit_message.replace("{max_homes}", &max_homes.to_string());
        player.send_message(&formatted_message);
        return;
    }
    
    // Check detailed configuration
    if let Some((_, _, _, _, _, min_distance, max_distance, allow_combat, _)) = detailed_config {
        // Check if player is in combat
        if !allow_combat && player.is_in_combat() {
            let combat_message = config_manager.get_error_message("in_combat", 
                "&cYou cannot set a home while in combat").await;
            player.send_message(&combat_message);
            return;
        }
        
        // Check distance from spawn
        let spawn_distance = get_distance_from_spawn(player.position);
        if spawn_distance < min_distance {
            let distance_message = config_manager.get_error_message("too_close_to_spawn", 
                "&cYou cannot set a home this close to spawn").await;
            player.send_message(&distance_message);
            return;
        }
    }
    
    // Set the home
    set_player_home(player, home_name, position).await;
    
    // Use configured success message
    let formatted_message = success_message.replace("{home_name}", &home_name);
    player.send_message(&formatted_message);
}
```

## 🎯 **Configuration Methods Available**

### **Direct Access Methods**

```rust
// Plugin information
let (name, version, description) = config_manager.get_plugin_info().await;

// General settings
let (language, debug, prefix) = config_manager.get_general_settings().await;

// Command configurations
let (cooldown, max_distance, cross_world, timeout) = config_manager.get_teleport_config().await;
let (max_homes, cooldown, cross_world) = config_manager.get_home_config_main().await;
let (cooldown, enabled) = config_manager.get_spawn_config().await;
let (max_warps, cooldown, cross_world) = config_manager.get_warp_config_main().await;

// Database configuration
let (db_type, sqlite_path) = config_manager.get_database_config().await;

// Utility methods
let debug_enabled = config_manager.is_debug_enabled().await;
let prefix = config_manager.get_prefix().await;
```

### **Configuration Section Methods**

```rust
// Get main configuration section
let main_section = config_manager.get_main_configuration_section().await;

// Get file-specific sections
let home_section = config_manager.get_home_configuration_section().await;
let warp_section = config_manager.get_warp_configuration_section().await;

// Get detailed configurations
let detailed_home = config_manager.get_home_config_detailed().await;
let detailed_warp = config_manager.get_warp_config_detailed().await;
```

### **Message System**

```rust
// Get messages by category and key
let message = config_manager.get_message("success", "home_teleport", "&aWelcome home!").await;

// Convenience methods
let success_msg = config_manager.get_success_message("home_teleport", "&aWelcome home!").await;
let error_msg = config_manager.get_error_message("home_not_found", "&cHome not found!").await;
let info_msg = config_manager.get_info_message("teleport_request_sent", "&eTeleport request sent").await;
```

### **Permissions and Lists**

```rust
// Get permissions by role
let admin_perms = config_manager.get_permissions("admin").await;
let player_perms = config_manager.get_permissions("player").await;

// Get any list from configuration
let commands_list = config_manager.auto_manager.get_string_list("allowed_commands").await;
```

## 🔧 **Configuration File Structure**

### **Main Config (`config.yml`)**
```yaml
plugin:
  name: "PumpkinEssentials"
  version: "1.0.0"
  description: "Essential commands and features for Pumpkin servers"

general:
  language: "en"
  debug: false
  prefix: "&8[&aEssentials&8]&r "

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

messages:
  success:
    home_teleport: "&aWelcome home!"
    home_set: "&aHome '{home_name}' set successfully!"
  
  error:
    home_not_found: "&cHome '{home_name}' not found!"
    home_limit: "&cYou have reached the maximum number of homes ({max_homes})"
    teleport_cooldown: "&cPlease wait before teleporting again"
    too_close_to_spawn: "&cYou cannot set a home this close to spawn"
    too_far_from_spawn: "&cYou cannot set a home this far from spawn"
    in_combat: "&cYou cannot set a home while in combat"
```

### **Home Config (`home/home_config.yml`)**
```yaml
home:
  max_homes: 3
  cooldown: 5
  cross_world: true
  allow_protected_areas: false
  allow_pvp_zones: true
  allow_nether: true
  allow_end: true
  min_distance_from_spawn: 100.0
  max_distance_from_spawn: 10000.0
  allow_in_combat: false
  allow_teleport_in_combat: false

home_names:
  default: "home"
  allowed_chars: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-"
  max_length: 16
  min_length: 1
```

## 🎉 **Benefits of the New System**

### **1. Familiar API**
- ✅ **Bukkit-like syntax** - Easy to learn for Java developers
- ✅ **Dot notation paths** - Intuitive navigation
- ✅ **Default values** - Safe access to missing values

### **2. Type Safety**
- ✅ **Compile-time checking** - Rust's type system prevents errors
- ✅ **Proper types** - No runtime type casting needed
- ✅ **Option handling** - Explicit handling of missing values

### **3. Performance**
- ✅ **Async/await** - Non-blocking configuration access
- ✅ **Cached access** - Fast repeated access to values
- ✅ **Efficient parsing** - YAML parsed once and cached

### **4. Flexibility**
- ✅ **Multiple files** - Access different configuration files
- ✅ **Nested sections** - Deep navigation in complex configs
- ✅ **List support** - Access lists of various types

## 🚀 **Testing the System**

You can test all the new features by calling:

```rust
let config_manager = EssentialsConfigManager::new(data_folder);
config_manager.test_configuration_features().await;
```

This will output a comprehensive test of all configuration features, showing:

1. **Direct config access** - Plugin info, general settings
2. **Command configurations** - Teleport, home, spawn, warp settings
3. **Configuration sections** - Bukkit-style section access
4. **File-specific sections** - Home and warp config files
5. **Detailed configurations** - All nested settings
6. **Message system** - Success, error, and info messages
7. **Database configuration** - Database type and paths
8. **Permissions** - Admin and player permissions
9. **Path existence** - Checking if paths exist
10. **List access** - Accessing permission lists

The new `ConfigurationSection` system provides a powerful, type-safe, and familiar way to access configuration values in PumpkinEssentials! 🎯 