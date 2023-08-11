use std::{collections::VecDeque, sync::{Arc, Mutex}};

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub id: String,
    pub message_body: String
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueueTags {
    key: String,
    value: String,
}

#[derive(Debug, Clone)]
pub struct Queue {
    name: String,
    tags: Vec<QueueTags>,

    messages: VecDeque<Message>,
}

/// Queue is a FIFO data structure to implement a mock SQS queue.
impl Queue {
    pub fn new(name: &str, tags: Vec<QueueTags>) -> Queue {
        Queue {
            name: name.to_owned(),
            tags,
            messages: VecDeque::new(),
        }
    }

    pub fn push(&mut self, msg: Message) {
        self.messages.push_back(msg);
    }

    /// Remove a message from the queue by id.
    pub fn remove(&mut self, id: &str) {
        self.messages
            .iter()
            .position(|message| message.id == id)
            .map(|index| self.messages.remove(index));
    }

    pub fn pop(&mut self) -> Option<Message> {
        self.messages.pop_front()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_push() {
        let mut queue = Queue::new("test", vec![]);
        let message = Message {
            id: "id".to_owned(),
        };
        queue.push(message.clone());
        assert_eq!(queue.messages.len(), 1);
        assert_eq!(queue.messages[0].id, "id");
    }

    #[test]
    fn test_remove() {
        let mut queue = Queue::new("test", vec![]);
        let message = Message {
            id: "id".to_owned(),
        };
        queue.push(message.clone());
        queue.remove("id");
        assert_eq!(queue.messages.len(), 0);
    }

    #[test]
    fn test_pop() {
        let mut queue = Queue::new("test", vec![]);
        let message = Message {
            id: "id".to_owned(),
        };
        queue.push(message.clone());
        let popped = queue.pop();
        assert_eq!(&popped, &Some(message));
        assert_eq!(queue.messages.len(), 0);
    }
}
