use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type MagicStore = Arc<RwLock<HashMap<String, String>>>;
