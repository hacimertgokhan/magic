use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type MagicStore = Arc<RwLock<HashMap<String, String>>>;
