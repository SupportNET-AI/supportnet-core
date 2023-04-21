use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::client::Client;

use crate::communication_channel::CommunicationChannel;


// Focus on managing the discord client
pub struct DiscordManager {
    discord_client: Client,
}

impl DiscordManager {
    pub async fn new() -> Self {
        DiscordManager {
            discord_client: Client::new("token", Handler),
        }
    }

    pub async fn start(&self) {
        self.discord_client.start().await;
    }

    pub async fn stop(&self) {
        self.discord_client.shutdown();
    }
}

impl CommunicationChannel for DiscordManager {
    async fn send_message(&self, recipient: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Implementation for sending a message via Discord
    }

    async fn receive_message(&self) -> Result<Message, Box<dyn std::error::Error>> {
        // Implementation for receiving a message from Discord
    }
}