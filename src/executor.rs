use crate::enums::MagicCommand;
use crate::types::MagicStore;

pub fn execute_command(store: &MagicStore, cmd: MagicCommand) {
    match cmd {
        MagicCommand::Summon { key, value } => {
            store.write().unwrap().insert(key, value.clone());
            println!("{}", value);
        }
        MagicCommand::Conjure { key } => {
            if let Some(value) = store.read().unwrap().get(&key) {
                println!("{}", value);
            } else {
                println!("Unknown incantation: {}", key);
            }
        }
        MagicCommand::Dispel { key } => {
            store.write().unwrap().remove(&key);
        }
        MagicCommand::Unknown => {
            println!("Unknown command.");
        }
    }
}
