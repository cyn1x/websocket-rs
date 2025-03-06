use std::sync::{Arc, Mutex, OnceLock, mpsc};

pub struct Event {
    sender: mpsc::Sender<String>,
    receiver: Mutex<mpsc::Receiver<String>>,
}

impl Event {
    fn new() -> Arc<Self> {
        let (sender, receiver) = mpsc::channel();
        Arc::new(Self {
            sender,
            receiver: Mutex::new(receiver),
        })
    }

    // Global static instance
    pub fn instance() -> Arc<Self> {
        static INSTANCE: OnceLock<Arc<Event>> = OnceLock::new();
        INSTANCE.get_or_init(|| Self::new()).clone()
    }

    pub fn send_msg(&self, msg: String) {
        let _ = self.sender.send(msg).expect("Error sending message");
    }

    pub fn recv_msg(&self) -> String {
        self.receiver
            .lock()
            .unwrap()
            .recv()
            .expect("Error receiving message")
    }
}
