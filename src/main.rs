#[cfg(test)]
mod tests;


use std::env;
use chrono::{DateTime, Utc, Timelike};
use chrono_tz::Tz;
use dotenvy::dotenv;
use serenity::client::Client;
use serenity::model::prelude::Channel;
use tokio::sync::{Mutex};
use std::sync::Arc;
use serenity::{
    async_trait,
    model::{
        channel::Message, 
        gateway::Ready,
        id::UserId,
    },
    prelude::*,
};


const SECONDS_IN_MINUTE: i64 = 60;
#[allow(dead_code)]
const SECONDS_IN_HOUR: i64 = 3600;
#[allow(dead_code)]
const SECONDS_IN_DAY: i64 = 86400;
#[allow(dead_code)]
const SECONDS_IN_WEEK: i64 = 604800;
const CHECK_IN_TIMEOUT: i64 = 150 * SECONDS_IN_MINUTE;

#[allow(dead_code)]
const EARLY_SOBRIETY_GUIDELINES: &str = "Consider the following guidelines for early sobriety (less than 3 months): \
    \n\n- Mood: Good -> Check-in timer: 1-2 times per day (e.g., 12h or 8h) \
    \n- Mood: Moderate -> Check-in timer: 2-3 times per day (e.g., 6h or 4h) \
    \n- Mood: Low -> Check-in timer: 3-4 times per day (e.g., 4h, 3h, or 2h) ";

#[allow(dead_code)]
const MID_TERM_SOBRIETY_GUIDELINES: &str = "Consider the following guidelines for mid-term sobriety (3 months to 1 year): \
    \n\n- Mood: Good -> Check-in timer: Every 1-2 days (e.g., 1d, 1d 12h) \
    \n- Mood: Moderate -> Check-in timer: Every day (e.g., 24h) \
    \n- Mood: Low -> Check-in timer: Twice per day (e.g., 12h) ";

#[allow(dead_code)]
const LONG_TERM_SOBRIETY_GUIDELINES: &str = "Consider the following guidelines for long-term sobriety (1 year and beyond): \
    \n\n- Mood: Good -> Check-in timer: Every 3-7 days (e.g., 3d, 5d, 1w) \
    \n- Mood: Moderate -> Check-in timer: Every 1-3 days (e.g., 1d, 2d, 3d) \
    \n- Mood: Low -> Check-in timer: Every day (e.g., 24h) ";


pub struct Handler {
    support_net: Arc<SupportNET>,
}


impl Handler {
    pub fn new(support_net: Arc<SupportNET>) -> Self {
        Self { support_net }
    }
}


#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        println!("Message received: {} from {}", msg.content, msg.author.name);
    
        // Ignore bot's own messages
        if msg.author.bot {
            println!("Message from bot. Ignoring.");
            return;
        }
    
        // Check if the message is a direct message
        if let Ok(channel) = msg.channel(&context).await {
            if let Channel::Private(_) = channel {
                // Handle the direct message here
                if msg.content.starts_with("!end") {
                    // end_conversation functionality goes here
                    let response = "Conversation ended. We'll check in with you later.";
                    if let Err(why) = msg.channel_id.say(&context.http, response).await {
                        println!("Error sending message: {:?}", why);
                    }
                    return;
                }
    
                if msg.author.id == self.support_net.user_id {
                    let support_net = &self.support_net;
                    let conversation_state = support_net.conversation_state.lock().await;
            
                    if !conversation_state.in_conversation {
                        drop(conversation_state); // Drop the MutexGuard before calling start_conversation()
                        support_net.start_conversation().await;
                    } else {
                        drop(conversation_state); // Drop the MutexGuard before calling reset_timeout_counter()
                        support_net.reset_timeout_counter().await;
                    }
    
                    #[allow(unused_variables)]
                    let user_message: ChatMessage = ChatMessage {
                        role: "user".to_string(),
                        content: msg.content.clone(),
                    };
    
                    // handle_user_message functionality goes here, it should be async and properly called.
                }
            }
        }
    }


    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}


pub struct SupportNetConfig {
    pub user_id: UserId,
    pub user_name: String,
    pub user_timezone: chrono_tz::Tz,
    pub user_sobriety_date: DateTime<chrono_tz::Tz>,
}


pub fn config_from_env() -> SupportNetConfig {
    let user_id_str = env::var("DISCORD_USER_ID").expect("Expected DISCORD_USER_ID in the environment");
    let user_id: UserId = user_id_str.parse::<UserId>().expect("Invalid DISCORD_USER_ID");

    let user_name = env::var("USER_NAME").expect("Expected USER_NAME in the environment");

    let user_timezone_str = env::var("USER_TIMEZONE").expect("Expected USER_TIMEZONE in the environment");
    let user_timezone = user_timezone_str.parse::<chrono_tz::Tz>().expect("Invalid timezone");

    let user_sobriety_date_str = env::var("USER_SOBRIETY_DATE").expect("Expected USER_SOBRIETY_DATE in the environment");
    let user_sobriety_date = DateTime::parse_from_rfc3339(&user_sobriety_date_str)
        .expect("Invalid sobriety date")
        .with_timezone(&user_timezone);

    SupportNetConfig {
        user_id,
        user_name,
        user_timezone,
        user_sobriety_date,
    }
}


#[allow(dead_code)]
pub struct SupportNET {
    discord_client: Mutex<Option<Arc<Mutex<Client>>>>,
    conversation_state: Mutex<ConversationState>,
    user_id: UserId,
    user_name: String,
    user_timezone: chrono_tz::Tz,
    user_sobriety_date: chrono::DateTime<chrono_tz::Tz>,
}


impl SupportNET {
    pub async fn new(config: SupportNetConfig, token: &str) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let intents = GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

        let support_net = Arc::new(Self {
            user_id: config.user_id,
            user_name: config.user_name,
            user_timezone: config.user_timezone,
            user_sobriety_date: config.user_sobriety_date,
            conversation_state: Mutex::new(ConversationState::default()),
            discord_client: Mutex::new(None),
        });

        let handler = Handler::new(Arc::clone(&support_net));
        let discord_client = Client::builder(token, intents)
            .event_handler(handler)
            .await?;

        support_net.set_discord_client(discord_client).await;

        Ok(support_net)
    }


    pub async fn set_discord_client(&self, discord_client: Client) {
        let discord_client = Arc::new(Mutex::new(discord_client));
        let mut discord_client_mutex = self.discord_client.lock().await;
        *discord_client_mutex = Some(discord_client);
    }
    

    pub async fn reset_timeout_counter(&self) {
        let mut conversation_state = self.conversation_state.lock().await;
        println!("Conversation already in progress. Resetting timeout counter.");
        conversation_state.timeout_counter = 0;
    }


    async fn start_conversation(&self) {
        let mut conversation_state = self.conversation_state.lock().await;
        conversation_state.in_conversation = true;
        conversation_state.timeout_counter = 0;
        println!("Conversation started.");
    }


    #[allow(dead_code)]
    async fn end_conversation(&self) {
        let new_check_in_timer = self.request_new_check_in_timeout().await;
    
        let mut conversation_state = self.conversation_state.lock().await;
    
        conversation_state.in_conversation = false;
        conversation_state.timeout_counter = 0;
        conversation_state.message_history.clear();
        conversation_state.check_in_timer = new_check_in_timer;
    
        println!("Conversation ended.");
    }
   

    #[allow(dead_code)]
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
    #[allow(dead_code)]
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


    /// Returns a prompt message for check-in time based on the user's sobriety duration.
    ///
    /// # Arguments
    ///
    /// * `sobriety_duration_days` - The number of days the user has been sober.
    #[allow(dead_code)]
    fn get_check_in_time_prompt(&self, sobriety_duration_days: i32) -> String {
        let mut prompt = format!("Based on the user's sobriety duration ({} days) and mood, determine an appropriate check-in timer for {} in addiction recovery. ",
                                 sobriety_duration_days, self.user_name);

        if sobriety_duration_days < 90 {
            prompt.push_str(EARLY_SOBRIETY_GUIDELINES);
        } else if sobriety_duration_days < 365 {
            prompt.push_str(MID_TERM_SOBRIETY_GUIDELINES);
        } else {
            prompt.push_str(LONG_TERM_SOBRIETY_GUIDELINES);
        }

        prompt.push_str("\n\nPlease provide your recommendation for a check-in timer ONLY in the format 'Xw Xd Xh Xm' where X is a number and w, d, h, and m stand for weeks, days, hours, and minutes, respectively. \
            You may leave out any time unit that is not needed. Do not include any additional text in your response.");

        prompt
    }


    /// Calculates and returns the user's sobriety duration in days.
    ///
    /// # Arguments
    ///
    /// * `reference_time` - (Optional) The reference time to calculate the sobriety duration from. If not provided, it defaults to the current time.
    ///
    /// # Examples
    ///
    /// ```
    /// let sobriety_duration_days = support_net.get_sobriety_duration_days(None);
    /// assert_eq!(sobriety_duration_days, 15);
    /// ```
    ///
    /// ```
    /// let reference_time = chrono_tz::America::Chicago.ymd(2023, 4, 17).and_hms(0, 0, 0);
    /// let sobriety_duration_days = support_net.get_sobriety_duration_days(Some(reference_time));
    /// assert_eq!(sobriety_duration_days, 20);
    /// ```
    #[allow(dead_code)]
    fn get_sobriety_duration_days(&self, reference_time: Option<DateTime<Tz>>) -> i32 {
        let sobriety_duration = self.get_sobriety_duration(reference_time);
        sobriety_duration.split_whitespace().next().unwrap().parse::<i32>().unwrap()
    }
}


#[allow(dead_code)]
struct ConversationState {
    in_conversation: bool,
    message_history: Vec<ChatMessage>,
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


#[allow(dead_code)]
struct ChatMessage {
    role: String,
    content: String,
}


#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment");
    let config = config_from_env();
    let support_net = SupportNET::new(config, &token).await.expect("Failed to initialize SupportNET");

    if let Some(client) = support_net
        .discord_client
        .lock()
        .await
        .as_ref() {
            if let Err(why) = client
                .clone()
                .lock()
                .await
                .start()
                .await {
                println!("An error occurred while running the client: {:?}", why);
            }
        } else {
            println!("Discord client not initialized.");
        };
}
