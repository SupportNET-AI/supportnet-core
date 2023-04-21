use crate::communication_channel::CommunicationChannel;

pub struct Support {
    communication_channel: Box<dyn CommunicationChannel>,
    // ...
}

// Focus on handling core logic and state of support system
impl Support {
    pub fn new(communication_channel: Box<dyn CommunicationChannel>) -> Self {
        Self {
            communication_channel,
            // ...
        }
    }

    fn process_check_in(&self) {
        println!("Check in");
    }
}