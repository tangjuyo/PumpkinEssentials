use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use std::collections::HashMap;
use std::fs;
use pumpkin_world::item::ItemStack;
use pumpkin_world::inventory::Inventory;
use serde_yaml::{Mapping, Sequence, Value};

use crate::models::{Kit, KitDataFile};

pub struct KitManager {
    data_path: PathBuf,
    data: Arc<Mutex<KitDataFile>>,
}

impl KitManager {
    pub async fn new(data_folder: PathBuf) -> Result<Self> {
        let data_path = data_folder.join("kits").join("data.yml");
        
        // Créer le dossier s'il n'existe pas
        if let Some(parent) = data_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let data = KitDataFile::load(&data_path)?;
        
        Ok(KitManager {
            data_path,
            data: Arc::new(Mutex::new(data)),
        })
    }

    pub fn parse_kit_item(item_string: &str) -> Result<String> {
        // Parse un item de kit et retourne une chaîne formatée
        let parts: Vec<&str> = item_string.split_whitespace().collect();
        
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty item string"));
        }
        
        let item_id = parts[0];
        let mut result = item_id.to_string();
        
        // Ajouter la quantité si spécifiée
        if parts.len() > 1 {
            if let Ok(quantity) = parts[1].parse::<i32>() {
                result.push_str(&format!(" {}", quantity));
            }
        }
        
        // Ajouter les propriétés si spécifiées
        if parts.len() > 2 {
            let properties = &parts[2..];
            result.push_str(&format!(" {}", properties.join(" ")));
        }
        
        Ok(result)
    }

    pub async fn create_kit(
        &self,
        kit_name: String,
        creator_uuid: Uuid,
        player_inventory: &dyn Inventory,
    ) -> Result<()> {
        let mut data = self.data.lock().await;
        
        // Vérifier si le kit existe déjà
        if data.kits.contains_key(&kit_name) {
            return Err(anyhow::anyhow!("Kit '{}' already exists", kit_name));
        }
        
        let mut kit_items = Vec::new();
        let now = chrono::Utc::now().to_rfc3339();
        
        // Parcourir l'inventaire du joueur
        for slot in 0..player_inventory.size() {
            let stack = player_inventory.get_stack(slot).await;
            let stack_guard = stack.lock().await;
            if !stack_guard.is_empty() {
                let item_id = stack_guard.item.registry_key.to_string();
                let quantity = stack_guard.item_count;
                
                let item_string = format!("{} {}", item_id, quantity);
                kit_items.push(item_string);
            }
        }
        
        let kit = Kit {
            name: kit_name.clone(),
            creator: creator_uuid.to_string(),
            created_at: now.clone(),
            updated_at: now,
            items: kit_items,
            cooldown: None, // Pas de cooldown par défaut
            permission: None, // Pas de permission par défaut
        };
        
        data.kits.insert(kit_name, kit);
        data.save(&self.data_path)?;
        
        Ok(())
    }

    pub async fn give_kit(
        &self,
        kit_name: &str,
        player_uuid: Uuid,
        player_inventory: &dyn Inventory,
        player_name: &str,
        server: &pumpkin::server::Server,
    ) -> Result<bool> {
        let mut data = self.data.lock().await;
        
        // Vérifier si le kit existe
        let kit = match data.kits.get(kit_name) {
            Some(kit) => kit.clone(),
            None => return Ok(false),
        };
        
        // Vérifier le cooldown
        let player_uuid_str = player_uuid.to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if let Some(cooldown) = kit.cooldown {
            if let Some(player_cooldowns) = data.player_cooldowns.get(&player_uuid_str) {
                if let Some(last_used) = player_cooldowns.get(kit_name) {
                    if now - last_used < cooldown {
                        return Ok(false);
                    }
                }
            }
        }
        
        // Mettre à jour le cooldown
        data.player_cooldowns
            .entry(player_uuid_str.clone())
            .or_insert_with(HashMap::new)
            .insert(kit_name.to_string(), now);
        
        // Donner les items/commandes du kit
        for item_string in &kit.items {
            if item_string.starts_with('/') {
                // C'est une commande
                let processed_command = item_string[1..].replace("{USERNAME}", player_name);
                
                let dispatcher = server.command_dispatcher.read().await;
                let mut sender = pumpkin::command::CommandSender::Console;
                dispatcher.handle_command(&mut sender, server, &processed_command).await;
                log::info!("Kit command executed: {}", processed_command);
            } else {
                // C'est un item vanilla
                let parts: Vec<&str> = item_string.split_whitespace().collect();
                log::info!("Processing kit item: '{}', parts: {:?}", item_string, parts);
                
                if parts.len() >= 2 {
                    let item_id = parts[0];
                    let quantity = parts[1].parse::<i32>().unwrap_or(1);
                    
                    log::info!("Looking for item: '{}' with quantity: {}", item_id, quantity);
                    
                    // Créer l'item
                    if let Some(item) = pumpkin_data::item::Item::from_registry_key(item_id) {
                        log::info!("Found item: {}", item_id);
                        let mut stack = ItemStack::new(quantity as u8, item);
                        
                        // Ajouter les propriétés si spécifiées
                        if parts.len() > 2 {
                            // TODO: Implémenter les propriétés d'items
                        }
                        
                        // Essayer d'ajouter l'item à l'inventaire
                        let mut added = false;
                        for slot in 0..player_inventory.size() {
                            let slot_stack = player_inventory.get_stack(slot).await;
                            let slot_guard = slot_stack.lock().await;
                            if slot_guard.is_empty() {
                                drop(slot_guard);
                                player_inventory.set_stack(slot, stack).await;
                                added = true;
                                log::info!("Added item {} to slot {}", item_id, slot);
                                break;
                            }
                        }
                        
                        if !added {
                            log::warn!("Could not add item {} to inventory (inventory full)", item_id);
                        }
                    } else {
                        log::warn!("Unknown item: {}", item_id);
                    }
                } else {
                    log::warn!("Invalid item format: '{}'", item_string);
                }
            }
        }
        
        data.save(&self.data_path)?;
        Ok(true)
    }

    pub async fn list_kits(&self) -> Result<Vec<String>> {
        let data = self.data.lock().await;
        Ok(data.kits.keys().cloned().collect())
    }

    pub async fn kit_exists(&self, kit_name: &str) -> Result<bool> {
        let data = self.data.lock().await;
        Ok(data.kits.contains_key(kit_name))
    }

    pub async fn delete_kit(&self, kit_name: &str) -> Result<bool> {
        let mut data = self.data.lock().await;
        if data.kits.remove(kit_name).is_some() {
            data.save(&self.data_path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_kit_info(&self, kit_name: &str) -> Result<Option<Kit>> {
        let data = self.data.lock().await;
        Ok(data.kits.get(kit_name).cloned())
    }

    pub async fn export_kits_to_external_format(&self, file_path: &str) -> Result<()> {
        let data = self.data.lock().await;
        data.save(file_path)?;
        Ok(())
    }

    pub async fn add_external_kit(&self, kit_name: &str, kit_data: &serde_yaml::Value) -> Result<()> {
        let mut data = self.data.lock().await;
        let now = chrono::Utc::now().to_rfc3339();
        
        let mut kit_items = Vec::new();
        
        if let Some(items) = kit_data.get("items").and_then(|v| v.as_sequence()) {
            for item in items {
                if let Some(item_str) = item.as_str() {
                    kit_items.push(item_str.to_string());
                }
            }
        }
        
        let delay = kit_data.get("delay")
            .and_then(|v| v.as_i64())
            .map(|d| if d < 0 { None } else { Some(d as u64) })
            .flatten();
        
        let kit = Kit {
            name: kit_name.to_string(),
            creator: "external".to_string(),
            created_at: now.clone(),
            updated_at: now,
            items: kit_items,
            cooldown: delay,
            permission: None,
        };
        
        data.kits.insert(kit_name.to_string(), kit);
        data.save(&self.data_path)?;
        
        Ok(())
    }
}

// Global access functions
pub async fn init_kit_manager(data_folder: PathBuf) -> Result<()> {
    let manager = KitManager::new(data_folder).await?;
    set_kit_manager(Arc::new(manager)).await;
    Ok(())
}

static KIT_MANAGER: once_cell::sync::Lazy<Arc<Mutex<Option<Arc<KitManager>>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

pub async fn get_kit_manager() -> Result<Arc<KitManager>> {
    let manager_guard = KIT_MANAGER.lock().await;
    manager_guard
        .as_ref()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Kit manager not initialized"))
}

pub async fn set_kit_manager(manager: Arc<KitManager>) {
    let mut manager_guard = KIT_MANAGER.lock().await;
    *manager_guard = Some(manager);
} 