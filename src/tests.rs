use chrono::TimeZone;
use chrono_tz::America;
use std::sync::Mutex;

use crate::SupportNET;
use crate::{ConversationState, CHECK_IN_TIMEOUT};

#[test]
fn test_get_sobriety_duration() {
    let reference_time = America::Chicago.with_ymd_and_hms(2023, 4, 10, 6, 45, 0).unwrap();

    // Define a list of test cases
    let test_cases = vec![
        (America::Chicago.with_ymd_and_hms(2023, 4, 9, 6, 45, 0).unwrap(), "1 day and 0 hours"),
        (America::Chicago.with_ymd_and_hms(2023, 4, 8, 6, 45, 0).unwrap(), "2 days and 0 hours"),
        (America::Chicago.with_ymd_and_hms(2023, 4, 10, 1, 45, 0).unwrap(), "0 days and 5 hours"),
    ];

    for (sobriety_date, expected_output) in test_cases {
        let mut support_net = SupportNET {
            discord_client: None,
            conversation_state: Mutex::new(ConversationState {
                check_in_timer: CHECK_IN_TIMEOUT,
                in_conversation: false,
                message_history: vec![],
                timeout_counter: 0,
            }),
            user_is_sober: true,
            user_name: "Alice".to_string(),
            user_timezone: America::Chicago,
            user_sobriety_date: sobriety_date,
        };

        let sobriety_duration = support_net.get_sobriety_duration(Some(reference_time));
        assert_eq!(sobriety_duration, expected_output);
    }
}
