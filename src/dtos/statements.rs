#[derive(Hash, PartialEq, Eq)]
pub enum PreparedStatementsKey {
    User(UserStatements),
    Championship(ChampionshipStatements),
    EventData(EventDataStatements),
}

#[derive(Hash, PartialEq, Eq)]
pub enum UserStatements {
    Insert,
    Delete,
    Activate,
    Deactivate,
    ById,
    ByEmail,
    EmailByEmail,
}

#[derive(Hash, PartialEq, Eq)]
pub enum ChampionshipStatements {
    Insert,
    Ports,
    ById,
    ByUser,
    Delete,
    NameByName
}

#[derive(Hash, PartialEq, Eq)]
pub enum EventDataStatements {
    Select,
    Insert,
    Update,
    Info
}
