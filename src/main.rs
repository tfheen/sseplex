extern crate actix_web;
extern crate tokio_timer;
extern crate uuid;
use futures::{Async, Poll};
use actix_web::{http, server, middleware, App, HttpResponse, HttpRequest, Error};
use actix_web::http::{StatusCode};
use bytes::{Bytes};
use futures::stream::poll_fn;
use std::thread;
use std::sync::{Arc};
use std::sync::RwLock;
use uuid::Uuid;

struct ClientState {
    task: futures::task::Task,
    events: Vec<String>,
    id: Uuid,
}

struct AppState {
    counter: Arc<RwLock<usize>>,
    clients: Arc<RwLock<Vec<ClientState>>>,
}

impl Drop for  {
    fn drop(&mut self) {
        println!("Dropping!");
    }
}


fn sse(req: &HttpRequest<AppState>) -> HttpResponse {
    let my_id = Uuid::new_v4();
    let counter = Arc::clone(&req.state().counter);
    let me = ClientState{task: futures::task::current(),
                         events: vec![],
                         id: my_id,
    };
    let mut t = req.state().clients.write().unwrap();
    t.push(me);
    let clients = Arc::clone(&req.state().clients);
 
    let server_events = poll_fn(move || -> Poll<Option<Bytes>, Error> {
        println!("poll");
        let counter = counter.read().unwrap();
        println!("counter.read: {:?}", counter);
        let mut clients = clients.write().unwrap();
        for client in clients.iter_mut() {
            if client.id == my_id {
                match client.events.pop() {
                    None => return Ok(Async::NotReady),
                    Some(evt) => {
                        let payload = format!("data: {}\n\n", evt);
                        return Ok(Async::Ready(Some(Bytes::from(payload))))
                    }
                }
            }
        }
        return Ok(Async::NotReady)
    });

    HttpResponse::build(StatusCode::OK)
        .content_type("text/event-stream")
        .streaming(server_events)
}

fn new_app(counter: Arc<RwLock<usize>>, clients: Arc<RwLock<Vec<ClientState>>>) -> App<AppState> {
    let state = AppState {
        counter: counter,
        clients: clients,
    };

    App::with_state(state)
        .middleware(middleware::Logger::default())
        .resource("/{topic}", |r| r.method(http::Method::GET).f(sse))
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let c = Arc::new(RwLock::new(10));
    let clients: Arc<RwLock<Vec<ClientState>>> = Arc::new(RwLock::new(vec![]));
    let val = Arc::clone(&c);
    let t_c = Arc::clone(&clients);

    thread::spawn(move || {
        loop {
            println!("before sleep");
            thread::sleep(::std::time::Duration::from_secs(1));
            println!("after sleep");
            let mut c = val.write().unwrap();
            println!("got c: {:?}", *c);
            *c += 1;
            println!("increased c: {:?}", *c);
            let mut notify_clients = t_c.write().unwrap();
            for t in notify_clients.iter_mut() {
                t.events.push(format!("foo {}", c));
                println!("State: {:?} {:#?}", t.id, t.events);
                t.task.notify();
            }
        }
    });

    server::new(move || new_app(c.clone(), clients.clone()))
        .bind("127.0.0.1:8080").unwrap()
        .run();
}
