pub(crate) mod nats;

pub(crate) trait PubSubClient {
    fn subscribe(subject: &Subject);
    fn publish(subject: &Subject, msg: &Message);
}

pub(crate) struct Subject(str);
pub(crate) struct Message(str);


