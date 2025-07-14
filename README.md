# PumpkinEssentials Plugin

A comprehensive collection of essential commands for Pumpkin Minecraft servers.

## Features

### Information

- ⚠️ = command coded or partially coded but isn't working yet
- ✅ = command available and working 
- ❌ = command not done yet

### Home System

- ⚠️ `/home [name]` - Teleport to your home
- ⚠️ `/sethome [name]` - Set a home at your current location  
- ⚠️ `/delhome <name>` - Delete a home
- ⚠️ `/renamehome <old> <new>` - Rename a home

### Gamemode Shortcuts

- ✅ `/gmc [player]` - Switch to Creative mode
- ✅ `/gms [player]` - Switch to Survival mode
- ✅ `/gma [player]` - Switch to Adventure mode
- ✅ `/gmsp [player]` - Switch to Spectator mode

### Teleportation

- ⚠️ `/back` - Return to your previous location before teleportation
- ⚠️ `/top` - Teleport to the highest block at your position
- ⚠️ `/tpa <player>` - Request to teleport to a player
- ⚠️ `/tpaccept` - Accept a teleport request
- ⚠️ `/tpdeny` - Deny a teleport request
- ⚠️ `/tpahere <player>` - Request a player to teleport to you
- ⚠️ `/tpall` - Teleport all players to you

### Utility Commands

- ✅ `/heal [player]` - Heal yourself or another player
- ✅ `/feed [player]` - Feed yourself or another player
- ✅ `/fly [player]` - Toggle flight mode
- ✅ `/god [player]` - Toggle god mode
- ✅ `/ping [player]` - Check ping
- ❌ `/repair` - Repair the item in your hand
- ✅ `/suicide` - Commit suicide
- ❌ `/killall` - Kill all entities
- ❌ `/enderchest [player]` - Open an enderchest
- ❌ `/ignore <player>` - Ignore a player
- ❌ `/kickall` - Kick all players
- ⚠️ `/sudo <player> <command>` - Execute a command as another player
- ✅ `/speed <walk|fly> <value> [player]` - Set walk or fly speed

## Next focus

- Some commentary are still in french ( due to my first language being french ). I will take the time to convert them all to english.
- finding and solving the teleportation issue to make every command that need teleporting available
- starting a real yml files structure to make persistant data & configuration 
- whenever the yml structure is done ==> starting on ignore command & on chat control with configuration for banning word

## Installation

This plugin is automatically loaded by Pumpkin when placed in the plugins directory.

## Author

Created and maintain by tangjuyo ( any PR are welcome ).

