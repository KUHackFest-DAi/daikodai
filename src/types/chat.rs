#[derive(Clone, Debug)]
pub enum ChatResponse {
    WelcomeMessage(String),

    BroadcastMessage(String),

    ListUser(String),

    AddNewUser(String),

    RemoveUser(String),
}

#[derive(Clone, Debug)]
pub enum ChatError {
    WelcomeMessage(String),

    BroadcastMessage(String),

    ListUser(String),

    AddNewUser(String),

    RemoveUser(String),
}
