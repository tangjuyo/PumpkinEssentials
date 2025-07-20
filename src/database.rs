use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct HomeLocation {
    pub player_uuid: String,
    pub home_name: String,
    pub world_name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DataFile {
    pub homes: Vec<HomeLocation>,
}

impl DataFile {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            let content = fs::read_to_string(path)?;
            let data: DataFile = serde_yaml::from_str(&content)?;
            Ok(data)
        } else {
            Ok(DataFile::default())
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}