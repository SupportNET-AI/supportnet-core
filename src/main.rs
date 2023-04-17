#[cfg(test)]
mod tests;


use std::sync::{Arc, Mutex};
use std::env;
use chrono::prelude::*;
use chrono::{DateTime, Utc, Duration, Timelike};
use chrono_tz::Tz;
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


const SECONDS_IN_MINUTE: i64 = 60;
const SECONDS_IN_HOUR: i64 = 3600;
const SECONDS_IN_DAY: i64 = 86400;
const SECONDS_IN_WEEK: i64 = 604800;
const CHECK_IN_TIMEOUT: i64 = 150 * SECONDS_IN_MINUTE;

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


pub struct SupportNetConfig {
    pub user_name: String,
    pub user_is_sober: bool,
    pub user_timezone: chrono_tz::Tz,
    pub user_sobriety_date: DateTime<chrono_tz::Tz>,
}


pub fn config_from_env() -> SupportNetConfig {
    let user_name = env::var("USER_NAME").expect("Expected USER_NAME in the environment");

    let user_is_sober_str = env::var("USER_IS_SOBER").expect("Expected USER_IS_SOBER in the environment");
    let user_is_sober = user_is_sober_str.parse::<bool>().expect("Invalid user is sober value");

    let user_timezone_str = env::var("USER_TIMEZONE").expect("Expected USER_TIMEZONE in the environment");
    let user_timezone = user_timezone_str.parse::<chrono_tz::Tz>().expect("Invalid timezone");

    let user_sobriety_date_str = env::var("USER_SOBRIETY_DATE").expect("Expected USER_SOBRIETY_DATE in the environment");
    let user_sobriety_date = DateTime::parse_from_rfc3339(&user_sobriety_date_str)
        .expect("Invalid sobriety date")
        .with_timezone(&user_timezone);

    SupportNetConfig {
        user_name,
        user_is_sober,
        user_timezone,
        user_sobriety_date,
    }
}


struct SupportNET {
    pub discord_client: Option<Client>,
    conversation_state: Mutex<ConversationState>,
    user_is_sober: bool,
    user_name: String,
    user_timezone: chrono_tz::Tz,
    user_sobriety_date: chrono::DateTime<chrono_tz::Tz>,
}


impl SupportNET {
    pub async fn new(config: SupportNetConfig, token: &str) -> Result<Self, Box<dyn std::error::Error>> {
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
            user_name: config.user_name,
            user_is_sober: config.user_is_sober,
            user_timezone: config.user_timezone,
            user_sobriety_date: config.user_sobriety_date,
            conversation_state,
            discord_client: Some(discord_client),
        })
    }


    fn start_conversation(&self) {
        let mut conversation_state = self.conversation_state.lock().unwrap();
        conversation_state.in_conversation = true;
        conversation_state.timeout_counter = 0;
        println!("Conversation started.");
    }


    async fn end_conversation(&self) {
        let new_check_in_timer = self.request_new_check_in_timeout().await;

        let mut conversation_state = self.conversation_state.lock().unwrap();

        conversation_state.in_conversation = false;
        conversation_state.timeout_counter = 0;
        conversation_state.message_history.clear();
        conversation_state.check_in_timer = new_check_in_timer;

        println!("Conversation ended.");
    }


    async fn request_new_check_in_timeout(&self) -> i64 {
        // TODO: Replace this with the actual logic for calculating the new check-in timer
        CHECK_IN_TIMEOUT
    }


    /// Returns a formatted string representing the duration of the user's sobriety.
    ///
    /// The duration is calculated based on the user's sobriety date and the current time,
    /// or the provided reference time, if given. The output format is: "{days} day(s) and {hours} hour(s)".
    ///
    /// # Arguments
    ///
    /// * `reference_time` - An optional reference time to calculate the sobriety duration.
    ///                      If not provided, the current time is used.
    ///
    /// # Examples
    ///
    /// ```
    /// let support_net = SupportNET::new();
    /// let sobriety_duration = support_net.get_sobriety_duration(None);
    /// println!("Sobriety duration: {}", sobriety_duration);
    /// ```
    fn get_sobriety_duration(&self, reference_time: Option<DateTime<Tz>>) -> String {
        // Use the provided reference time or the current time in the user's timezone.
        let localized_time = match reference_time {
            Some(time) => time,
            None => Utc::now().with_timezone(&self.user_timezone),
        };

        // Calculate the duration between the user's sobriety date and the reference or current time.
        let duration = localized_time.signed_duration_since(self.user_sobriety_date.clone().with_timezone(&self.user_timezone));
        
        // Determine the number of days and hours in the duration.
        let days = duration.num_days();
        let hours = (duration.num_seconds() % SECONDS_IN_DAY) / SECONDS_IN_HOUR;  // Convert remaining seconds to hours

        // Format the output string with appropriate pluralization.
        format!("{} day{} and {} hour{}",
                days,
                if days != 1 { "s" } else { "" },
                hours,
                if hours != 1 { "s" } else { "" })
    }


    pub fn get_user_current_time(&self) -> String {
        let localized_time = Utc::now().with_timezone(&self.user_timezone);
        localized_time.format("%H:%M:%S").to_string()
    }


    /// Check if the given `user_time` is outside the range specified by `start_hour` and `end_hour`.
    ///
    /// # Arguments
    ///
    /// * `user_time` - A `chrono::DateTime<chrono_tz::Tz>` object representing the user's local time.
    /// * `start_hour` - The starting hour of the time range (inclusive), in 24-hour format.
    /// * `end_hour` - The ending hour of the time range (exclusive), in 24-hour format.
    ///
    /// # Returns
    ///
    /// Returns `true` if the `user_time` is outside the specified range, `false` otherwise.
    pub fn is_time_outside_range(&self, user_time: chrono::DateTime<chrono_tz::Tz>, start_hour: u32, end_hour: u32) -> bool {
        let current_hour = user_time.hour();

        if start_hour <= end_hour {
            !(start_hour <= current_hour && current_hour < end_hour)
        } else {
            !((start_hour <= current_hour) || (current_hour < end_hour))
        }
    }
}


struct ConversationState {
    in_conversation: bool,
    message_history: Vec<AIMessage>,
    check_in_timer: i64,
    timeout_counter: i64,
}


impl Default for ConversationState {
    fn default() -> Self {
        Self {
            in_conversation: false,
            timeout_counter: 0,
            message_history: vec![],
            check_in_timer: CHECK_IN_TIMEOUT,
        }
    }
}


struct AIMessage {
    role: String,
    content: String,
}


#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment");
    let config = config_from_env();
    let mut support_net = SupportNET::new(config, &token).await.expect("Failed to initialize SupportNET");

    if let Err(why) = support_net.discord_client.as_mut().unwrap().start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
