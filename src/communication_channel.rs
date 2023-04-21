pub trait CommunicationChannel {
    async fn send_message(&self, recipient: &str, message: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn receive_message(&self) -> Result<Message, Box<dyn std::error::Error>>;
}
