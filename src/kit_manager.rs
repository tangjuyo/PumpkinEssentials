use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use pumpkin_world::item::ItemStack;
use pumpkin_world::inventory::Inventory;
use serde_yaml::{Mapping, Sequence, Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Kit {
    pub name: String,
    pub creator: String, // UUID du créateur
    pub created_at: String,
    pub updated_at: String,
    pub items: Vec<String>, // Format externe simple: "diamond_sword 1" ou "/command {USERNAME}"
    pub cooldown: Option<u64>, // Cooldown en secondes, None = pas de cooldown
    pub permission: Option<String>, // Permission requise pour utiliser le kit
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct KitDataFile {
    pub kits: HashMap<String, Kit>,
    pub player_cooldowns: HashMap<String, HashMap<String, u64>>, // player_uuid -> kit_name -> last_used_timestamp
}

impl KitDataFile {
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            let content = fs::read_to_string(path)?;
            let external_data = serde_yaml::from_str::<serde_yaml::Value>(&content)?;
            Self::load_from_external_format(external_data)
        } else {
            Ok(KitDataFile::default())
        }
    }

    /// Charge depuis le format externe
    fn load_from_external_format(external_data: serde_yaml::Value) -> Result<Self> {
        let mut data = KitDataFile::default();
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(kits) = external_data.get("kits").and_then(|v| v.as_mapping()) {
            for (kit_name, kit_data) in kits {
                if let Some(kit_name_str) = kit_name.as_str() {
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
                        name: kit_name_str.to_string(),
                        creator: "external".to_string(),
                        created_at: now.clone(),
                        updated_at: now.clone(),
                        items: kit_items,
                        cooldown: delay,
                        permission: None,
                    };

                    data.kits.insert(kit_name_str.to_string(), kit);
                }
            }
        }

        Ok(data)
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        // Sauvegarder au format externe
        let mut external_data = Mapping::new();
        let mut kits_mapping = Mapping::new();

        for (kit_name, kit) in &self.kits {
            let mut kit_data = Mapping::new();
            
            // Convertir le cooldown au format externe
            let delay_value = match kit.cooldown {
                Some(cooldown) => Value::Number(serde_yaml::Number::from(cooldown as i64)),
                None => Value::Number(serde_yaml::Number::from(-1)),
            };
            kit_data.insert(
                Value::String("delay".to_string()),
                delay_value
            );

            // Convertir les items au format externe
            let mut items_array = Sequence::new();
            for item in &kit.items {
                items_array.push(Value::String(item.clone()));
            }
            
            kit_data.insert(
                Value::String("items".to_string()),
                Value::Sequence(items_array)
            );

            kits_mapping.insert(
                Value::String(kit_name.clone()),
                Value::Mapping(kit_data)
            );
        }

        external_data.insert(
            Value::String("kits".to_string()),
            Value::Mapping(kits_mapping)
        );

        let content = serde_yaml::to_string(&external_data)?;
        fs::write(path, content)?;
        Ok(())
    }
}

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

    /// Parse un item au format string (comme dans votre exemple)
    pub fn parse_kit_item(item_string: &str) -> Result<String> {
        let item_string = item_string.trim();
        
        // Vérifier si c'est une commande (commence par /)
        if item_string.starts_with('/') {
            return Ok(item_string.to_string());
        }
        
        // Parser un item vanilla
        let parts: Vec<&str> = item_string.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty item string"));
        }
        
        let item_id = parts[0].to_string();
        let count = if parts.len() > 1 {
            parts[1].parse::<u8>().unwrap_or(1)
        } else {
            1
        };
        
        let mut item_string = format!("{} {}", item_id.replace("minecraft:", ""), count);
        
        // Parser les propriétés supplémentaires
        for i in 2..parts.len() {
            let part = parts[i];
            if part.contains(':') {
                let (key, value) = part.split_once(':').unwrap();
                match key {
                    "durability" => {
                        item_string.push_str(&format!(" durability:{}", value));
                    }
                    "name" => {
                        item_string.push_str(&format!(" name:{}", value));
                    }
                    _ => {
                        // Autres propriétés sont considérées comme des enchantements
                        if let Ok(_level) = value.parse::<u8>() {
                            item_string.push_str(&format!(" {}:{}", key, value));
                        }
                    }
                }
            }
        }
        
        Ok(item_string)
    }

    /// Crée un kit à partir de l'inventaire actuel du joueur
    pub async fn create_kit(
        &self,
        kit_name: String,
        creator_uuid: Uuid,
        player_inventory: &dyn Inventory,
    ) -> Result<()> {
        let mut data = self.data.lock().await;
        let creator_uuid_str = creator_uuid.to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Convertir l'inventaire en items de kit au format externe
        let mut kit_items = Vec::new();

        // Parcourir tous les slots de l'inventaire
        for slot in 0..player_inventory.size() {
            let stack = player_inventory.get_stack(slot).await;
            let stack_guard = stack.lock().await;
            if !stack_guard.is_empty() {
                // Créer une représentation au format externe
                let item_string = format!("{} {}", 
                    stack_guard.item.registry_key.to_string().replace("minecraft:", ""),
                    stack_guard.item_count
                );
                
                kit_items.push(item_string);
            }
        }

        let kit = Kit {
            name: kit_name.clone(),
            creator: creator_uuid_str,
            created_at: now.clone(),
            updated_at: now,
            items: kit_items,
            cooldown: None, // Par défaut, pas de cooldown
            permission: None, // Par défaut, pas de permission spéciale
        };

        data.kits.insert(kit_name, kit);
        data.save(&self.data_path)?;

        Ok(())
    }

    /// Donne le kit à un joueur
    pub async fn give_kit(
        &self,
        kit_name: &str,
        player_uuid: Uuid,
        player_inventory: &dyn Inventory,
        player_name: &str,
        server: &pumpkin::server::Server,
    ) -> Result<bool> {
        let mut data = self.data.lock().await;
        let player_uuid_str = player_uuid.to_string();

        // Vérifier si le kit existe et le cloner
        let kit = match data.kits.get(kit_name) {
            Some(k) => k.clone(),
            None => {
                return Err(anyhow::anyhow!("Kit '{}' does not exist", kit_name));
            }
        };

        // Vérifier les permissions
        if let Some(_permission) = &kit.permission {
            // TODO: Implémenter la vérification des permissions
            // Pour l'instant, on ignore
        }

        // Vérifier le cooldown
        if let Some(cooldown_seconds) = kit.cooldown {
            let now = chrono::Utc::now().timestamp() as u64;
            let player_cooldowns = data.player_cooldowns
                .entry(player_uuid_str.clone())
                .or_insert_with(HashMap::new);

            if let Some(last_used) = player_cooldowns.get(kit_name) {
                let time_since_last_use = now - last_used;
                if time_since_last_use < cooldown_seconds {
                    let remaining = cooldown_seconds - time_since_last_use;
                    return Err(anyhow::anyhow!(
                        "Kit '{}' is on cooldown. Please wait {} seconds.",
                        kit_name,
                        remaining
                    ));
                }
            }

            // Mettre à jour le timestamp d'utilisation
            player_cooldowns.insert(kit_name.to_string(), now);
        }

        // Donner les items au joueur
        for item_string in &kit.items {
            // Vérifier si c'est une commande
            if item_string.starts_with('/') {
                // Remplacer {USERNAME} par le nom du joueur et enlever le slash
                let processed_command = item_string[1..].replace("{USERNAME}", player_name);
                
                // Exécuter la commande via le serveur
                let dispatcher = server.command_dispatcher.read().await;
                let mut sender = pumpkin::command::CommandSender::Console;
                
                // Debug: vérifier si la commande existe
                let command_name = processed_command.split_whitespace().next().unwrap_or("");
                
                // Exécuter la commande en tant que console (avec tous les droits)
                dispatcher.handle_command(&mut sender, server, &processed_command).await;
            } else {
                // Parser l'item vanilla
                match Self::parse_kit_item(item_string) {
                    Ok(parsed_item) => {
                        // Extraire l'item ID et le count
                        let parts: Vec<&str> = parsed_item.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let item_id = parts[0];
                            let count: u8 = parts[1].parse().unwrap_or(1);
                            let registry_key = item_id; // Sans le préfixe minecraft:
                            let found = pumpkin_data::item::Item::from_registry_key(&registry_key);
                            if let Some(item) = found {
                                let item_stack = ItemStack::new(count, item);
                                
                                // Essayer d'ajouter l'item à l'inventaire
                                let mut added = false;
                                for slot in 0..player_inventory.size() {
                                    let stack = player_inventory.get_stack(slot).await;
                                    let stack_guard = stack.lock().await;
                                    if stack_guard.is_empty() {
                                        drop(stack_guard);
                                        player_inventory.set_stack(slot, item_stack).await;
                                        added = true;
                                        break;
                                    }
                                }
                                
                                if !added {
                                    // Si l'inventaire est plein, on arrête
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to parse item '{}': {}", item_string, e);
                    }
                }
            }
        }

        data.save(&self.data_path)?;
        Ok(true)
    }

    /// Liste tous les kits disponibles
    pub async fn list_kits(&self) -> Result<Vec<String>> {
        let data = self.data.lock().await;
        Ok(data.kits.keys().cloned().collect())
    }

    /// Vérifie si un kit existe
    pub async fn kit_exists(&self, kit_name: &str) -> Result<bool> {
        let data = self.data.lock().await;
        Ok(data.kits.contains_key(kit_name))
    }

    /// Supprime un kit
    pub async fn delete_kit(&self, kit_name: &str) -> Result<bool> {
        let mut data = self.data.lock().await;
        let removed = data.kits.remove(kit_name).is_some();
        if removed {
            data.save(&self.data_path)?;
        }
        Ok(removed)
    }

    /// Obtient les informations d'un kit
    pub async fn get_kit_info(&self, kit_name: &str) -> Result<Option<Kit>> {
        let data = self.data.lock().await;
        Ok(data.kits.get(kit_name).cloned())
    }

    /// Exporte tous les kits au format externe
    pub async fn export_kits_to_external_format(&self, file_path: &str) -> Result<()> {
        let data = self.data.lock().await;
        data.save(file_path)?;
        Ok(())
    }

    /// Ajoute un kit externe au système
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

// Fonction d'initialisation globale
pub async fn init_kit_manager(data_folder: PathBuf) -> Result<()> {
    let _manager = KitManager::new(data_folder).await?;
    Ok(())
}

// Variable globale pour accéder au gestionnaire de kits
static KIT_MANAGER: once_cell::sync::Lazy<Arc<Mutex<Option<Arc<KitManager>>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

pub async fn get_kit_manager() -> Result<Arc<KitManager>> {
    let manager_guard = KIT_MANAGER.lock().await;
    manager_guard.as_ref()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Kit manager not initialized"))
}

pub async fn set_kit_manager(manager: Arc<KitManager>) {
    let mut manager_guard = KIT_MANAGER.lock().await;
    *manager_guard = Some(manager);
} 