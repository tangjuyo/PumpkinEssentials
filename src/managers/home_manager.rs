use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use pumpkin_util::math::vector3::Vector3;
// Configuration trait not needed for current implementation

use crate::config::EssentialsConfigManager;
use crate::models::{DataFile, HomeLocation};

pub struct HomeManager {
    data_path: PathBuf,
    config_manager: EssentialsConfigManager,
    // Mutex pour accès concurrent au fichier YAML
    data: Arc<Mutex<DataFile>>,
}

impl HomeManager {
    pub async fn new(data_folder: PathBuf) -> Result<Self> {
        let data_path = data_folder.join("home").join("data.yml");
        // Charge le fichier YAML (ou crée vide)
        let data = DataFile::load(&data_path)?;
        let config_manager = EssentialsConfigManager::new(data_folder);
        
        // Initialize the configuration
        config_manager.init().await?;

        Ok(HomeManager {
            data_path,
            config_manager,
            data: Arc::new(Mutex::new(data)),
        })
    }

    pub async fn set_home(
        &self,
        player_uuid: Uuid,
        home_name: String,
        world_name: String,
        position: Vector3<f64>,
        yaw: f32,
        pitch: f32,
    ) -> Result<()> {
        let mut data = self.data.lock().await;
        let player_uuid_str = player_uuid.to_string();

        // Vérifie la limite de homes using direct YAML access
        let (max_homes, _, _) = self.config_manager.get_home_config_main().await;
        
        // Get or create player's home list
        let player_homes = data.homes.entry(player_uuid_str.clone()).or_insert_with(Vec::new);
        
        // Check if home already exists
        let existing_home_index = player_homes.iter().position(|h| h.home_name == home_name);
        
        // Check home limit (only if adding a new home)
        if existing_home_index.is_none() && player_homes.len() >= max_homes as usize {
            return Err(anyhow::anyhow!("Home limit reached"));
        }

        // Create the new home
        let home = HomeLocation {
            home_name: home_name.clone(),
            world_name,
            x: position.x,
            y: position.y,
            z: position.z,
            yaw,
            pitch,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        // Replace existing home or add new one
        if let Some(index) = existing_home_index {
            player_homes[index] = home;
        } else {
            player_homes.push(home);
        }

        // Sauvegarde dans le fichier YAML
        println!("Sauvegarde dans {:?}", self.data_path);
        data.save(&self.data_path)?;
        println!("Sauvegarde terminée");

        Ok(())
    }

    pub async fn get_home(&self, player_uuid: Uuid, home_name: &str) -> Result<Option<(String, Vector3<f64>, f32, f32)>> {
        let data = self.data.lock().await;
        let player_uuid_str = player_uuid.to_string();
        
        if let Some(player_homes) = data.homes.get(&player_uuid_str) {
            if let Some(home) = player_homes.iter().find(|h| h.home_name == home_name) {
                return Ok(Some((
                    home.world_name.clone(),
                    Vector3::new(home.x, home.y, home.z),
                    home.yaw,
                    home.pitch,
                )));
            }
        }
        Ok(None)
    }

    pub async fn get_player_homes(&self, player_uuid: Uuid) -> Result<Vec<(String, Vector3<f64>, f32, f32)>> {
        let data = self.data.lock().await;
        let player_uuid_str = player_uuid.to_string();
        
        if let Some(player_homes) = data.homes.get(&player_uuid_str) {
            Ok(player_homes
                .iter()
                .map(|h| (
                    h.home_name.clone(),
                    Vector3::new(h.x, h.y, h.z),
                    h.yaw,
                    h.pitch,
                ))
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn delete_home(&self, player_uuid: Uuid, home_name: &str) -> Result<bool> {
        let mut data = self.data.lock().await;
        let player_uuid_str = player_uuid.to_string();
        
        if let Some(player_homes) = data.homes.get_mut(&player_uuid_str) {
            let before = player_homes.len();
            player_homes.retain(|h| h.home_name != home_name);
            let deleted = before != player_homes.len();
            
            // Remove player entry if no homes left
            if player_homes.is_empty() {
                data.homes.remove(&player_uuid_str);
            }
            
            if deleted {
                data.save(&self.data_path)?;
            }
            Ok(deleted)
        } else {
            Ok(false)
        }
    }

    pub async fn delete_all_player_homes(&self, player_uuid: Uuid) -> Result<usize> {
        let mut data = self.data.lock().await;
        let player_uuid_str = player_uuid.to_string();
        
        if let Some(player_homes) = data.homes.remove(&player_uuid_str) {
            let deleted = player_homes.len();
            if deleted > 0 {
                data.save(&self.data_path)?;
            }
            Ok(deleted)
        } else {
            Ok(0)
        }
    }

    pub async fn get_home_count(&self, player_uuid: Uuid) -> Result<usize> {
        let data = self.data.lock().await;
        let player_uuid_str = player_uuid.to_string();
        
        if let Some(player_homes) = data.homes.get(&player_uuid_str) {
            Ok(player_homes.len())
        } else {
            Ok(0)
        }
    }

    pub async fn home_exists(&self, player_uuid: Uuid, home_name: &str) -> Result<bool> {
        let data = self.data.lock().await;
        let player_uuid_str = player_uuid.to_string();
        
        if let Some(player_homes) = data.homes.get(&player_uuid_str) {
            Ok(player_homes.iter().any(|h| h.home_name == home_name))
        } else {
            Ok(false)
        }
    }

    pub fn get_config_manager(&self) -> &EssentialsConfigManager {
        &self.config_manager
    }
}

// Global home manager instance
lazy_static::lazy_static! {
    pub static ref HOME_MANAGER: Arc<Mutex<Option<Arc<HomeManager>>>> = Arc::new(Mutex::new(None));
}

/// Initialize the global home manager
pub async fn init_home_manager(data_folder: PathBuf) -> Result<()> {
    let home_manager = Arc::new(HomeManager::new(data_folder).await?);
    
    let mut global_manager = HOME_MANAGER.lock().await;
    *global_manager = Some(home_manager);
    
    log::info!("[HomeManager] Global home manager initialized successfully");
    Ok(())
}

/// Get the global home manager
pub async fn get_home_manager() -> Result<Arc<HomeManager>> {
    let global_manager = HOME_MANAGER.lock().await;
    global_manager
        .as_ref()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Home manager not initialized"))
}
