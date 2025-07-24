use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use std::fs;
use serde_yaml::{Mapping, Value};

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
            let mut items_sequence = serde_yaml::Sequence::new();
            for item in &kit.items {
                items_sequence.push(Value::String(item.clone()));
            }
            kit_data.insert(
                Value::String("items".to_string()),
                Value::Sequence(items_sequence)
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