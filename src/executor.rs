use crate::types::MagicStore;
use crate::enums::MagicCommand;

pub async fn execute_command(store: &MagicStore, cmd: MagicCommand) -> String {
    match cmd {
        MagicCommand::Summon { key, value } => {
            store.write().await.insert(key.clone(), value.clone());
            format!("Summoned: {}", value)
        }
        MagicCommand::Conjure { key } => {
            let store_read = store.read().await;
            match store_read.get(&key) {
                Some(value) => format!("{}", value),
                None => format!("Unknown incantation: {}", key),
            }
        }
        MagicCommand::Dispel { key } => {
            store.write().await.remove(&key);
            format!("Dispelled: {}", key)
        }
        MagicCommand::Unknown => "Unknown command.".to_string(),
    }
}


pub async fn executor_command_string_parser(store: MagicStore, command: String) -> String {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();

    if parts.len() >= 4 && parts[0].eq_ignore_ascii_case("SUMMON") && parts[2].eq_ignore_ascii_case("AS") {
        let key = parts[1].to_string();
        let value = parts[3..].join(" ");
        execute_command(&store, MagicCommand::Summon { key, value }).await
    } else if parts.len() >= 2 && parts[0].eq_ignore_ascii_case("CONJURE") {
        let key = parts[1].to_string();
        execute_command(&store, MagicCommand::Conjure { key }).await
    } else if parts.len() >= 2 && parts[0].eq_ignore_ascii_case("DISPEL") {
        let key = parts[1].to_string();
        execute_command(&store, MagicCommand::Dispel { key }).await
    } else {
        execute_command(&store, MagicCommand::Unknown).await
    }
}
