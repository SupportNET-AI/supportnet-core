pub struct ConversationManager {
    conversation_state: ConversationState,
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