use pumpkin_util::math::vector3::Vector3;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// Global storage for player homes
lazy_static::lazy_static! {
    pub static ref PLAYER_HOMES: Arc<Mutex<HashMap<Uuid, HashMap<String, (Vector3<f64>, f32, f32)>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub const ARG_HOME_NAME: &str = "name";
