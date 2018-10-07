extern crate actix;

use actix::prelude::*;
use std::collections::{HashMap};
use std::time::{Duration};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);

pub struct EventSource {
    sessions: HashMap<usize, Recipient<SSEEvent>>,
    max_client_id: usize,
    counter: usize,
}

#[derive(Message)]
#[derive(Debug)]
pub struct SSEEvent {
    pub topic: String,
    pub text: String,
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<SSEEvent>,
}

#[derive(Message)]
pub struct Disconnect {
    pub id: usize,
}

impl Default for EventSource {
    fn default() -> EventSource {
        EventSource {
            sessions: HashMap::new(),
            max_client_id: 0,
            counter: 0,
        }
    }
}

impl Actor for EventSource {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Starting eventsource");
        self.hb(ctx);
    }
}

impl Handler<Connect> for EventSource {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Connect");

        // register session with random id
        self.max_client_id += 1;
        let id = self.max_client_id;
        self.sessions.insert(id, msg.addr);

        // send id back
        id
    }
}

impl Handler<Disconnect> for EventSource {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Disconnecting {}", msg.id);
        self.sessions.remove(&msg.id);
    }
}

impl EventSource {
    fn hb(&self, ctx: &mut Context<Self>) {
        println!("hb");
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, _ctx| {
            act.generate_data();
        });
    }

    fn generate_data(&mut self) {
        println!("hb tick {}", self.counter);
        self.counter += 1;
        let mut dead_sessions = Vec::new();
        for (id, addr) in self.sessions.iter_mut() {
            println!("Sending to {}", id);
            match addr.try_send(SSEEvent { topic: "foo".to_string(), text: format!("event {}", self.counter), }) {
                Ok(_) => (),
                Err(SendError::Closed(_)) => { dead_sessions.push(*id); () },
                Err(_) => (),
            }
        }
        for id in dead_sessions {
            self.sessions.remove(&id);
        }
    }
}
