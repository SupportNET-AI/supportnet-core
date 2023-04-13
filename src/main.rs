use std::sync::{Arc, Mutex};
use std::env;
use chrono::{DateTime, Utc, Duration};
use dotenvy::dotenv;
use serenity::client::Client;
use std::collections::HashMap;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};


const SECONDS_IN_MINUTE: i32 = 60;
const SECONDS_IN_HOUR: i32 = 3600;
const SECONDS_IN_DAY: i32 = 86400;
const SECONDS_IN_WEEK: i32 = 604800;
const CHECK_IN_TIMEOUT: i32 = 150 * SECONDS_IN_MINUTE;


struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("{}: {}", msg.author.tag(), msg.content);
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.tag());
    }
}


struct SupportNET {
    discord_client: Client,
    conversation_state: Mutex<ConversationState>,
    user_timezone: chrono_tz::Tz,
    begin_quiet_hours: u32,
    end_quiet_hours: u32,
}


impl SupportNET {
    pub async fn new(token: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let intents = GatewayIntents::default();
        let discord_client = Client::builder(token, intents)
            .event_handler(Handler)
            .await?;

        let conversation_state = Mutex::new(ConversationState {
            in_conversation: false,
            message_history: vec![],
            check_in_timer: CHECK_IN_TIMEOUT,
            timeout_counter: 0,
        });

        let user_timezone = chrono_tz::US::Central;
        let begin_quiet_hours = 21;
        let end_quiet_hours = 7;

        Ok(Self {
            discord_client,
            conversation_state,
            user_timezone,
            begin_quiet_hours,
            end_quiet_hours,
        })
    }
}



struct ConversationState {
    in_conversation: bool,
    message_history: Vec<Message>,
    check_in_timer: i32,
    timeout_counter: i32,
}


struct AIMessage {
    role: String,
    content: String,
}



#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found in environment");

    let mut support_net = SupportNET::new(&token).await.expect("Error creating SupportNET");
    if let Err(why) = support_net.discord_client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
