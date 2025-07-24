use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use pumpkin_util::math::vector3::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnLocation {
    pub world_name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SpawnData {
    pub spawn: Option<SpawnLocation>,
}

pub struct SpawnManager {
    data_path: PathBuf,
    data: Arc<Mutex<SpawnData>>,
}

impl SpawnManager {
    pub async fn new(data_folder: PathBuf) -> Result<Self> {
        let data_path = data_folder.join("spawn.yml");
        let data = if data_path.exists() {
            let file = std::fs::File::open(&data_path)?;
            serde_yaml::from_reader(file).unwrap_or_default()
        } else {
            SpawnData::default()
        };
        Ok(SpawnManager {
            data_path,
            data: Arc::new(Mutex::new(data)),
        })
    }

    pub async fn set_spawn(&self, location: SpawnLocation) -> Result<()> {
        let mut data = self.data.lock().await;
        data.spawn = Some(location);
        let file = std::fs::File::create(&self.data_path)?;
        serde_yaml::to_writer(file, &*data)?;
        Ok(())
    }

    pub async fn get_spawn(&self) -> Option<SpawnLocation> {
        let data = self.data.lock().await;
        data.spawn.clone()
    }
}

// Global spawn manager instance
lazy_static::lazy_static! {
    pub static ref SPAWN_MANAGER: Arc<Mutex<Option<Arc<SpawnManager>>>> = Arc::new(Mutex::new(None));
}

/// Initialize the global spawn manager
pub async fn init_spawn_manager(data_folder: PathBuf) -> Result<()> {
    let spawn_manager = Arc::new(SpawnManager::new(data_folder).await?);
    let mut global_manager = SPAWN_MANAGER.lock().await;
    *global_manager = Some(spawn_manager);
    log::info!("[SpawnManager] Global spawn manager initialized successfully");
    Ok(())
}

/// Get the global spawn manager
pub async fn get_spawn_manager() -> Result<Arc<SpawnManager>> {
    let global_manager = SPAWN_MANAGER.lock().await;
    global_manager
        .as_ref()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Spawn manager not initialized"))
} 