use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::client::Client;

pub struct DiscordManager {
    discord_client: Client,
}

impl DiscordManager {
    pub async fn new() -> Self {
        DiscordManager {
            discord_client: Client::new("token", Handler),
        }
    }
}