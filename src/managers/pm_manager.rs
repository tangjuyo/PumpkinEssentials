use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PMData {
    pub last_senders: HashMap<String, String>, // UUID string -> UUID string
}

impl PMData {
    pub fn new() -> Self {
        Self {
            last_senders: HashMap::new(),
        }
    }

    pub fn load(data_path: &PathBuf) -> Result<Self> {
        if data_path.exists() {
            let content = std::fs::read_to_string(data_path)?;
            let data: PMData = serde_yaml::from_str(&content)?;
            Ok(data)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self, data_path: &PathBuf) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(data_path, content)?;
        Ok(())
    }
}

pub struct PMManager {
    data: Arc<Mutex<PMData>>,
    data_path: PathBuf,
}

impl PMManager {
    pub fn new(data_path: PathBuf) -> Result<Self> {
        let data = PMData::load(&data_path)?;
        Ok(Self {
            data: Arc::new(Mutex::new(data)),
            data_path,
        })
    }

    pub async fn get_last_sender(&self, player_uuid: Uuid) -> Option<Uuid> {
        let data = self.data.lock().await;
        let player_uuid_str = player_uuid.to_string();
        
        data.last_senders.get(&player_uuid_str)
            .and_then(|sender_str| sender_str.parse::<Uuid>().ok())
    }

    pub async fn set_last_sender(&self, player_uuid: Uuid, sender_uuid: Uuid) {
        let mut data = self.data.lock().await;
        let player_uuid_str = player_uuid.to_string();
        let sender_uuid_str = sender_uuid.to_string();
        
        data.last_senders.insert(player_uuid_str, sender_uuid_str);
        
        // Sauvegarder les données
        if let Err(e) = data.save(&self.data_path) {
            log::error!("Failed to save PM data: {}", e);
        }
    }

    pub async fn clear_last_sender(&self, player_uuid: Uuid) {
        let mut data = self.data.lock().await;
        let player_uuid_str = player_uuid.to_string();
        
        data.last_senders.remove(&player_uuid_str);
        
        // Sauvegarder les données
        if let Err(e) = data.save(&self.data_path) {
            log::error!("Failed to save PM data: {}", e);
        }
    }
}

// Variable globale pour le gestionnaire de PM
static PM_MANAGER: tokio::sync::OnceCell<Arc<PMManager>> = tokio::sync::OnceCell::const_new();

pub async fn init_pm_manager(data_folder: PathBuf) -> Result<()> {
    let data_path = data_folder.join("pm_data.yml");
    let pm_manager = PMManager::new(data_path)?;
    
    PM_MANAGER.set(Arc::new(pm_manager))
        .map_err(|_| anyhow::anyhow!("PM manager already initialized"))?;
    
    log::info!("[Essentials] PM manager initialized");
    Ok(())
}

pub async fn get_pm_manager() -> Arc<PMManager> {
    PM_MANAGER.get()
        .expect("PM manager not initialized")
        .clone()
} 