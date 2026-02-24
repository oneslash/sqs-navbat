use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: String,
    pub message_body: String,
    pub receipt_handle: Option<String>,
    pub receive_count: u32,
    pub visible_at: Instant,
    pub first_received_at: Option<Instant>,
}

impl Message {
    pub fn new(id: String, message_body: String) -> Self {
        Message {
            id,
            message_body,
            receipt_handle: None,
            receive_count: 0,
            visible_at: Instant::now(),
            first_received_at: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueueTags {
    key: String,
    value: String,
}

#[derive(Debug, Clone)]
pub struct Queue {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    tags: Vec<QueueTags>,
    pub default_visibility_timeout: u32,
    messages: VecDeque<Message>,
}

/// Queue is a FIFO data structure to implement a mock SQS queue.
impl Queue {
    pub fn new(name: &str, tags: Vec<QueueTags>, default_visibility_timeout: Option<u32>) -> Queue {
        Queue {
            name: name.to_owned(),
            tags,
            default_visibility_timeout: default_visibility_timeout.unwrap_or(30),
            messages: VecDeque::new(),
        }
    }

    pub fn push(&mut self, msg: Message) {
        self.messages.push_back(msg);
    }

    /// Receive up to `max_messages` visible messages from the queue.
    /// Messages are not removed — they become invisible for `visibility_timeout` seconds.
    /// Returns clones of the received messages (with receipt_handle set).
    pub fn receive(&mut self, max_messages: u32, visibility_timeout: Option<u32>) -> Vec<Message> {
        let timeout = visibility_timeout.unwrap_or(self.default_visibility_timeout);
        let now = Instant::now();
        let mut received = Vec::new();

        for msg in self.messages.iter_mut() {
            if received.len() >= max_messages as usize {
                break;
            }
            if msg.visible_at <= now {
                let handle = uuid::Uuid::new_v4().to_string();
                msg.receipt_handle = Some(handle);
                msg.receive_count += 1;
                msg.visible_at = now + std::time::Duration::from_secs(timeout as u64);
                if msg.first_received_at.is_none() {
                    msg.first_received_at = Some(now);
                }
                received.push(msg.clone());
            }
        }

        received
    }

    /// Delete a message by its receipt handle. Returns true if found and removed.
    pub fn delete_by_receipt_handle(&mut self, handle: &str) -> bool {
        if let Some(pos) = self
            .messages
            .iter()
            .position(|m| m.receipt_handle.as_deref() == Some(handle))
        {
            self.messages.remove(pos);
            true
        } else {
            false
        }
    }

    /// Change the visibility timeout of an in-flight message. Returns true if found.
    pub fn change_visibility(&mut self, handle: &str, timeout: u32) -> bool {
        if let Some(msg) = self
            .messages
            .iter_mut()
            .find(|m| m.receipt_handle.as_deref() == Some(handle))
        {
            msg.visible_at = Instant::now() + std::time::Duration::from_secs(timeout as u64);
            true
        } else {
            false
        }
    }

    /// Count of messages that are currently visible (available to receive).
    pub fn approximate_number_of_messages(&self) -> usize {
        let now = Instant::now();
        self.messages.iter().filter(|m| m.visible_at <= now).count()
    }

    /// Count of messages that are currently in-flight (not visible).
    pub fn approximate_number_of_messages_not_visible(&self) -> usize {
        let now = Instant::now();
        self.messages.iter().filter(|m| m.visible_at > now).count()
    }

    /// Remove a message from the queue by id.
    pub fn remove(&mut self, id: &str) {
        self.messages
            .iter()
            .position(|message| message.id == id)
            .map(|index| self.messages.remove(index));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_push() {
        let mut queue = Queue::new("test", vec![], None);
        let message = Message::new("id".to_owned(), "message_body".to_owned());
        queue.push(message);
        assert_eq!(queue.messages.len(), 1);
        assert_eq!(queue.messages[0].id, "id");
    }

    #[test]
    fn test_remove() {
        let mut queue = Queue::new("test", vec![], None);
        let message = Message::new("id".to_owned(), "message_body".to_owned());
        queue.push(message);
        queue.remove("id");
        assert_eq!(queue.messages.len(), 0);
    }

    #[test]
    fn test_receive_returns_visible_messages() {
        let mut queue = Queue::new("test", vec![], Some(30));
        queue.push(Message::new("1".to_owned(), "body1".to_owned()));
        queue.push(Message::new("2".to_owned(), "body2".to_owned()));

        let received = queue.receive(1, None);
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].id, "1");
        assert!(received[0].receipt_handle.is_some());
        assert_eq!(received[0].receive_count, 1);

        // Message should now be invisible — receiving again should get message 2
        let received2 = queue.receive(1, None);
        assert_eq!(received2.len(), 1);
        assert_eq!(received2[0].id, "2");
    }

    #[test]
    fn test_receive_respects_max_messages() {
        let mut queue = Queue::new("test", vec![], None);
        for i in 0..5 {
            queue.push(Message::new(i.to_string(), format!("body{}", i)));
        }
        let received = queue.receive(3, None);
        assert_eq!(received.len(), 3);
    }

    #[test]
    fn test_delete_by_receipt_handle() {
        let mut queue = Queue::new("test", vec![], None);
        queue.push(Message::new("1".to_owned(), "body".to_owned()));
        let received = queue.receive(1, None);
        let handle = received[0].receipt_handle.as_ref().unwrap().clone();

        assert!(queue.delete_by_receipt_handle(&handle));
        assert_eq!(queue.messages.len(), 0);
    }

    #[test]
    fn test_delete_by_receipt_handle_not_found() {
        let mut queue = Queue::new("test", vec![], None);
        assert!(!queue.delete_by_receipt_handle("nonexistent"));
    }

    #[test]
    fn test_change_visibility() {
        let mut queue = Queue::new("test", vec![], None);
        queue.push(Message::new("1".to_owned(), "body".to_owned()));
        let received = queue.receive(1, None);
        let handle = received[0].receipt_handle.as_ref().unwrap().clone();

        // Set visibility to 0 — should make it immediately visible again
        assert!(queue.change_visibility(&handle, 0));

        let received2 = queue.receive(1, None);
        assert_eq!(received2.len(), 1);
        assert_eq!(received2[0].id, "1");
        assert_eq!(received2[0].receive_count, 2);
    }

    #[test]
    fn test_approximate_counts() {
        let mut queue = Queue::new("test", vec![], None);
        queue.push(Message::new("1".to_owned(), "body1".to_owned()));
        queue.push(Message::new("2".to_owned(), "body2".to_owned()));

        assert_eq!(queue.approximate_number_of_messages(), 2);
        assert_eq!(queue.approximate_number_of_messages_not_visible(), 0);

        queue.receive(1, None);

        assert_eq!(queue.approximate_number_of_messages(), 1);
        assert_eq!(queue.approximate_number_of_messages_not_visible(), 1);
    }
}
