pub enum MagicCommand {
    Summon { key: String, value: String },
    Conjure { key: String },
    Dispel { key: String },
    SendTo { target: String, value: String },
    Unknown,
}