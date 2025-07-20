# SQLite Database System for PumpkinEssentials

## Overview

This document describes the SQLite database system implemented for the PumpkinEssentials plugin to provide persistent storage for player homes and other data.

## Architecture

The database system consists of three main components:

1. **DatabaseManager** (`src/database.rs`) - Low-level database operations
2. **HomeManager** (`src/home_manager.rs`) - High-level home management logic
3. **Commands** - Updated to use the database system

## Database Schema

### Homes Table

```sql
CREATE TABLE homes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_uuid TEXT NOT NULL,
    home_name TEXT NOT NULL,
    world_name TEXT NOT NULL,
    x REAL NOT NULL,
    y REAL NOT NULL,
    z REAL NOT NULL,
    yaw REAL NOT NULL,
    pitch REAL NOT NULL,
    created_at TEXT NOT NULL,
    last_used TEXT,
    UNIQUE(player_uuid, home_name)
);
```

**Fields:**
- `id` - Auto-incrementing primary key
- `player_uuid` - Player's UUID (TEXT format)
- `home_name` - Name of the home (e.g., "home", "spawn", "base")
- `world_name` - Name of the world where the home is located
- `x`, `y`, `z` - Position coordinates (REAL for precision)
- `yaw`, `pitch` - Player rotation (REAL for precision)
- `created_at` - ISO 8601 timestamp when home was created
- `last_used` - ISO 8601 timestamp when home was last used (nullable)

**Indexes:**
- `idx_homes_player_uuid` - Index on player_uuid for faster lookups
- Unique constraint on `(player_uuid, home_name)` prevents duplicate home names per player

## Configuration

The database path is configured in the main configuration file:

```yaml
database:
  type: "sqlite"
  sqlite:
    path: "essentials.db"
```

The database file will be created at `{plugin_data_folder}/essentials.db`.

## Usage

### Commands

The following commands have been updated to use the database system:

- `/sethome [name]` - Set a home at current location
- `/home [name]` - Teleport to a home
- `/delhome [name]` - Delete a home
- `/renamehome <old_name> <new_name>` - Rename a home
- `/homes` - List all homes for the current player

### API Usage

```rust
// Get the global home manager
let home_manager = get_home_manager().await?;

// Set a home
home_manager.set_home(
    player_uuid,
    "home".to_string(),
    "world".to_string(),
    position,
    yaw,
    pitch,
).await?;

// Get a home
let home = home_manager.get_home(player_uuid, "home").await?;

// Get all homes for a player
let homes = home_manager.get_player_homes(player_uuid).await?;

// Delete a home
let deleted = home_manager.delete_home(player_uuid, "home").await?;
```

## Features

### Persistent Storage
- Homes are stored in SQLite database and persist across server restarts
- Automatic database initialization and table creation
- Connection pooling for efficient database access

### Configuration Integration
- Respects home limits from configuration
- Uses configured messages for success/error responses
- Debug logging when enabled

### Error Handling
- Comprehensive error handling with detailed error messages
- Graceful fallbacks when database operations fail
- Proper cleanup on errors

### Performance
- Connection pooling with SQLx
- Indexed queries for fast lookups
- Async/await for non-blocking operations

## Migration from In-Memory Storage

The system has been migrated from in-memory HashMap storage to persistent SQLite storage:

**Before:**
```rust
static PLAYER_HOMES: Lazy<Arc<Mutex<HashMap<Uuid, HashMap<String, (Vector3<f64>, f32, f32)>>>>>
```

**After:**
```rust
// Database-backed storage with full persistence
let home_manager = get_home_manager().await?;
let homes = home_manager.get_player_homes(player_uuid).await?;
```

## Database Operations

### CRUD Operations

**Create:**
```rust
home_manager.set_home(player_uuid, name, world, position, yaw, pitch).await
```

**Read:**
```rust
// Get specific home
let home = home_manager.get_home(player_uuid, name).await

// Get all homes for player
let homes = home_manager.get_player_homes(player_uuid).await

// Check if home exists
let exists = home_manager.home_exists(player_uuid, name).await

// Get home count
let count = home_manager.get_home_count(player_uuid).await
```

**Update:**
```rust
// Update last used timestamp (automatic on teleport)
home_manager.get_home(player_uuid, name).await // Updates last_used

// Rename home (delete + create)
home_manager.delete_home(player_uuid, old_name).await
home_manager.set_home(player_uuid, new_name, world, position, yaw, pitch).await
```

**Delete:**
```rust
let deleted = home_manager.delete_home(player_uuid, name).await
```

### Statistics

```rust
let stats = home_manager.get_stats().await?;
println!("Total homes: {}", stats.total_homes);
println!("Unique players: {}", stats.unique_players);
println!("Average homes per player: {:.2}", stats.avg_homes_per_player);
```

## Dependencies

The following dependencies were added to `Cargo.toml`:

```toml
chrono = { version = "0.4", features = ["serde"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono"] }
```

## File Structure

```
src/
├── database.rs          # Database manager and operations
├── home_manager.rs      # High-level home management
├── commands/
│   ├── sethome.rs       # Updated to use database
│   ├── home.rs          # Updated to use database
│   ├── delhome.rs       # Updated to use database
│   ├── renamehome.rs    # Updated to use database
│   ├── homes.rs         # New command to list homes
│   └── home_common.rs   # Removed in-memory storage
└── lib.rs              # Updated to initialize database system
```

## Future Enhancements

1. **Warp System** - Extend database to store warps
2. **Player Data** - Store additional player data (economy, permissions, etc.)
3. **Backup System** - Automatic database backups
4. **Migration System** - Database schema versioning and migrations
5. **Performance Monitoring** - Query performance metrics
6. **Multi-Database Support** - Support for PostgreSQL/MySQL

## Troubleshooting

### Common Issues

1. **Database Connection Failed**
   - Check file permissions on plugin data folder
   - Ensure SQLite is available on the system
   - Check logs for detailed error messages

2. **Home Not Found**
   - Verify home name spelling
   - Check if player has any homes with `/homes`
   - Ensure database is properly initialized

3. **Permission Errors**
   - Check file permissions on database file
   - Ensure plugin has write access to data folder

### Debug Mode

Enable debug mode in configuration to see detailed database operations:

```yaml
general:
  debug: true
```

This will log:
- Database connection attempts
- SQL queries being executed
- Home operations (set, get, delete)
- Configuration loading

## Security Considerations

1. **SQL Injection Protection** - All queries use parameterized statements
2. **Input Validation** - Home names and coordinates are validated
3. **File Permissions** - Database file should have restricted permissions
4. **Data Integrity** - Unique constraints prevent duplicate entries

## Performance Considerations

1. **Connection Pooling** - SQLx manages connection pool automatically
2. **Indexes** - Player UUID is indexed for fast lookups
3. **Async Operations** - All database operations are non-blocking
4. **Batch Operations** - Consider batch operations for bulk data

## Testing

The database system can be tested with:

```bash
# Check compilation
cargo check

# Build the plugin
cargo build

# Run tests (when implemented)
cargo test
```

## Conclusion

The SQLite database system provides a robust, scalable, and persistent storage solution for the PumpkinEssentials plugin. It replaces the previous in-memory storage with a proper database that ensures data persistence and provides better performance for larger datasets. 