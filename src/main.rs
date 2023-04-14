use std::sync::{Arc, Mutex};
use std::env;
use chrono::{DateTime, Utc, Duration};
use dotenvy::dotenv;
use serenity::client::Client;
use serenity::model::prelude::Channel;
use std::collections::HashMap;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
    utils::MessageBuilder,
};


const SECONDS_IN_MINUTE: i32 = 60;
const SECONDS_IN_HOUR: i32 = 3600;
const SECONDS_IN_DAY: i32 = 86400;
const SECONDS_IN_WEEK: i32 = 604800;
const CHECK_IN_TIMEOUT: i32 = 150 * SECONDS_IN_MINUTE;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        println!("Message received: {} from {}", msg.content, msg.author.name);

        // Ignore bot's own messages
        if msg.author.bot {
            return;
        }

        // Check if the message is a direct message
        if let Ok(channel) = msg.channel(&context).await {
            if let Channel::Private(_) = channel {
                // Handle the direct message here
                // You can use the user ID with msg.author.id
                let response = MessageBuilder::new()
                    .push("Hello, ")
                    .push_bold_safe(&msg.author.name)
                    .push("! You sent me a direct message.")
                    .build();

                if let Err(why) = msg.channel_id.say(&context.http, &response).await {
                    println!("Error sending message: {:?}", why);
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}


struct SupportNET {
    discord_client: Client,
    conversation_state: Mutex<ConversationState>,
    user_is_sober: bool,
    user_name: String,
    user_timezone: chrono_tz::Tz,
    user_sobriety_date: chrono::DateTime<chrono_tz::Tz>,
}


impl SupportNET {
    pub async fn new(token: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let user_name = env::var("USER_NAME").expect("Expected USER_NAME in the environment");
        
        let user_is_sober_str = env::var("USER_IS_SOBER").expect("Expected USER_IS_SOBER in the environment");
        let user_is_sober = user_is_sober_str.parse::<bool>().expect("Invalid user is sober value");

        let user_timezone_str = env::var("USER_TIMEZONE").expect("Expected USER_TIMEZONE in the environment");
        let user_timezone = user_timezone_str.parse::<chrono_tz::Tz>().expect("Invalid timezone");
        
        let user_sobriety_date_str = env::var("USER_SOBRIETY_DATE").expect("Expected USER_SOBRIETY_DATE in the environment");
        let user_sobriety_date = DateTime::parse_from_rfc3339(&user_sobriety_date_str)
            .expect("Invalid sobriety date")
            .with_timezone(&user_timezone);

        
        let intents = GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
        let discord_client = Client::builder(token, intents)
            .event_handler(Handler)
            .await?;

        let conversation_state = Mutex::new(ConversationState {
            in_conversation: false,
            message_history: vec![],
            check_in_timer: CHECK_IN_TIMEOUT,
            timeout_counter: 0,
        });

        Ok(Self {
            discord_client,
            conversation_state,
            user_is_sober,
            user_name,
            user_timezone,
            user_sobriety_date,
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
