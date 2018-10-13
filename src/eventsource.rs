extern crate actix;

use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use std::time::{Duration};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);

pub struct EventSource {
    topics: HashMap<String, HashSet<Recipient<SSEEvent>>>,
    counter: usize,
}

#[derive(Message)]
#[derive(Debug)]
pub struct SSEEvent {
    pub topic: String,
    pub text: String,
}

#[derive(Message)]
pub struct Connect {
    pub topic: String,
    pub addr: Recipient<SSEEvent>,
}

#[derive(Message)]
pub struct Publish {
    pub topic: String,
    pub text: String,
}

#[derive(Message)]
pub struct Disconnect {
    pub addr: Recipient<SSEEvent>,
}

#[derive(Message)]
pub struct StartDummySender;

impl Default for EventSource {
    fn default() -> EventSource {
        EventSource {
            topics: HashMap::new(),
            counter: 0,
        }
    }
}

impl Actor for EventSource {
    type Context = Context<Self>;
}

impl Handler<Connect> for EventSource {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        info!("Client connecting");

        if !self.topics.contains_key(&msg.topic) {
            self.topics.insert(msg.topic.clone(), HashSet::new());
        }
        self.topics.get_mut(&msg.topic).unwrap().insert(msg.addr);
    }
}

impl Handler<Disconnect> for EventSource {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        let mut empty_topics = Vec::new();
        for (topic, subscribers) in self.topics.iter_mut() {
            subscribers.remove(&msg.addr);
            if subscribers.is_empty() {
                empty_topics.push(topic.clone());
            }
        }
        for e in empty_topics {
            self.topics.remove(&e);
        }

    }
}

impl Handler<StartDummySender> for EventSource {
    type Result = ();

    fn handle(&mut self, _msg: StartDummySender, ctx: &mut Context<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, _ctx| {
            act.generate_data();
        });
    }
}

impl Handler<Publish> for EventSource {
    type Result = ();

    fn handle(&mut self, msg: Publish, _: &mut Context<Self>) -> Self::Result {
        match self.topics.get(&msg.topic) {
            Some(subs) => {
                for sub in subs {
                    sub.try_send(SSEEvent { topic: msg.topic.clone(), text: msg.text.clone(), });
                }
            }
            _ => (), // Key missing, nothing to do,
        }
    }
}

impl EventSource {
    fn generate_data(&mut self) {
        debug!("Heartbeat tick {}", self.counter);
        for (topic, subs) in self.topics.iter() {
            self.counter += 1;
            debug!("Sending on {}", topic);
            for sub in subs {
                sub.try_send(SSEEvent { topic: topic.clone(), text: format!("event {}", self.counter), });
            }
        }
    }
}
