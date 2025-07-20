use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use pumpkin_util::math::vector3::Vector3;

use crate::config::EssentialsConfigManager;
use crate::database::{DataFile, HomeLocation};

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

        // Vérifie la limite de homes
        let (max_homes, _, _) = self.config_manager.get_home_config_main().await;
        let count = data.homes.iter().filter(|h| h.player_uuid == player_uuid.to_string()).count();
        if count >= max_homes as usize && !data.homes.iter().any(|h| h.player_uuid == player_uuid.to_string() && h.home_name == home_name) {
            return Err(anyhow::anyhow!("Home limit reached"));
        }

        // Supprime l'ancien home si il existe
        data.homes.retain(|h| !(h.player_uuid == player_uuid.to_string() && h.home_name == home_name));

        // Ajoute le nouveau home
        let home = HomeLocation {
            player_uuid: player_uuid.to_string(),
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
        data.homes.push(home);

        // Sauvegarde dans le fichier YAML
        println!("Sauvegarde dans {:?}", self.data_path);
        data.save(&self.data_path)?;
        println!("Sauvegarde terminée");

        Ok(())
    }

    pub async fn get_home(&self, player_uuid: Uuid, home_name: &str) -> Result<Option<(String, Vector3<f64>, f32, f32)>> {
        let data = self.data.lock().await;
        if let Some(home) = data.homes.iter().find(|h| h.player_uuid == player_uuid.to_string() && h.home_name == home_name) {
            Ok(Some((
                home.world_name.clone(),
                Vector3::new(home.x, home.y, home.z),
                home.yaw,
                home.pitch,
            )))
        } else {
            Ok(None)
        }
    }

    pub async fn get_player_homes(&self, player_uuid: Uuid) -> Result<Vec<(String, Vector3<f64>, f32, f32)>> {
        let data = self.data.lock().await;
        Ok(data.homes
            .iter()
            .filter(|h| h.player_uuid == player_uuid.to_string())
            .map(|h| (
                h.home_name.clone(),
                Vector3::new(h.x, h.y, h.z),
                h.yaw,
                h.pitch,
            ))
            .collect())
    }

    pub async fn delete_home(&self, player_uuid: Uuid, home_name: &str) -> Result<bool> {
        let mut data = self.data.lock().await;
        let before = data.homes.len();
        data.homes.retain(|h| !(h.player_uuid == player_uuid.to_string() && h.home_name == home_name));
        let deleted = before != data.homes.len();
        if deleted {
            data.save(&self.data_path)?;
        }
        Ok(deleted)
    }

    pub async fn delete_all_player_homes(&self, player_uuid: Uuid) -> Result<usize> {
        let mut data = self.data.lock().await;
        let before = data.homes.len();
        data.homes.retain(|h| h.player_uuid != player_uuid.to_string());
        let deleted = before - data.homes.len();
        if deleted > 0 {
            data.save(&self.data_path)?;
        }
        Ok(deleted)
    }

    pub async fn get_home_count(&self, player_uuid: Uuid) -> Result<usize> {
        let data = self.data.lock().await;
        Ok(data.homes.iter().filter(|h| h.player_uuid == player_uuid.to_string()).count())
    }

    pub async fn home_exists(&self, player_uuid: Uuid, home_name: &str) -> Result<bool> {
        let data = self.data.lock().await;
        Ok(data.homes.iter().any(|h| h.player_uuid == player_uuid.to_string() && h.home_name == home_name))
    }

    pub fn get_config_manager(&self) -> &EssentialsConfigManager {
        &self.config_manager
    }
}

/// Global home manager instance
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
