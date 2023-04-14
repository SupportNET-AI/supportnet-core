use chrono::DateTime;
use chrono::TimeZone;
use chrono_tz::America;
use std::sync::Mutex;
use tokio_test::block_on;


use crate::SupportNetConfig;
use crate::SupportNET;
use crate::{ConversationState, CHECK_IN_TIMEOUT, AIMessage};


async fn setup_test_support_net(user_sobriety_date: Option<DateTime<chrono_tz::Tz>>) -> SupportNET {
    let config = SupportNetConfig {
        user_name: "test_user".to_string(),
        user_is_sober: true,
        user_timezone: chrono_tz::America::Chicago,
        user_sobriety_date: user_sobriety_date.unwrap_or_else(|| {
            DateTime::parse_from_rfc3339("2023-04-06T00:00:00-05:00")
                .unwrap()
                .with_timezone(&chrono_tz::America::Chicago)
        }),
    };

    let token = "your_test_token";
    let support_net = SupportNET::new(config, token)
        .await
        .expect("Failed to initialize SupportNET for test");

    support_net
}


#[tokio::test]
async fn test_start_conversation() {
    let support_net = setup_test_support_net(None).await;

    {
        let conversation_state = support_net.conversation_state.lock().unwrap();
        assert_eq!(conversation_state.in_conversation, false);
    }

    support_net.start_conversation();

    {
        let conversation_state = support_net.conversation_state.lock().unwrap();
        assert_eq!(conversation_state.in_conversation, true);
        assert_eq!(conversation_state.timeout_counter, 0);
    }
}


#[tokio::test]
async fn test_end_conversation() {
    let mut support_net = setup_test_support_net(None).await;
    support_net.start_conversation();

    // Mimic an ongoing conversation
    {
        let mut conversation_state = support_net.conversation_state.lock().unwrap();
        conversation_state.timeout_counter = 2;
        conversation_state.message_history.push(AIMessage {
            role: "test_role".to_string(),
            content: "Test message".to_string(),
        });
    }

    // Call end_conversation
    support_net.end_conversation().await;

    // Assert conversation state has been reset
    {
        let conversation_state = support_net.conversation_state.lock().unwrap();
        assert_eq!(conversation_state.in_conversation, false);
        assert_eq!(conversation_state.timeout_counter, 0);
        assert!(conversation_state.message_history.is_empty());
        assert_eq!(conversation_state.check_in_timer, CHECK_IN_TIMEOUT);
    }
}


#[tokio::test]
async fn test_get_sobriety_duration() {
    let reference_time = America::Chicago.with_ymd_and_hms(2023, 4, 10, 6, 45, 0).unwrap();

    // Define a list of test cases
    let test_cases = vec![
        (America::Chicago.with_ymd_and_hms(2023, 4, 9, 6, 45, 0).unwrap(), "1 day and 0 hours"),
        (America::Chicago.with_ymd_and_hms(2023, 4, 8, 6, 45, 0).unwrap(), "2 days and 0 hours"),
        (America::Chicago.with_ymd_and_hms(2023, 4, 10, 1, 45, 0).unwrap(), "0 days and 5 hours"),
    ];

    for (sobriety_date, expected_output) in test_cases {
        let mut support_net = setup_test_support_net(Some(sobriety_date)).await;

        let sobriety_duration = support_net.get_sobriety_duration(Some(reference_time));
        assert_eq!(sobriety_duration, expected_output);
    }
}