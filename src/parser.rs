use crate::enums::MagicCommand;

pub fn parse_magic_command(input: &str) -> MagicCommand {
    let input = input.trim();

    if input.to_uppercase().starts_with("SUMMON ") {
        let rest = input[7..].trim();
        if let Some((key, value_part)) = rest.split_once(" AS ") {
            let value = value_part.trim().trim_matches('"').to_string();
            return MagicCommand::Summon {
                key: key.trim().to_string(),
                value,
            };
        }
    }

    if input.to_uppercase().starts_with("CONJURE ") {
        let key = input[8..].trim();
        return MagicCommand::Conjure {
            key: key.to_string(),
        };
    }

    if input.to_uppercase().starts_with("CONJURE ") {
        let key = input[8..].trim();
        return MagicCommand::Conjure {
            key: key.to_string(),
        };
    }

    if input.to_uppercase().starts_with("DISPEL ") {
        let key = input[7..].trim();
        return MagicCommand::Dispel {
            key: key.to_string(),
        };
    }

    MagicCommand::Unknown
}
