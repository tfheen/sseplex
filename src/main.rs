extern crate actix_web;
extern crate actix;
extern crate tokio_timer;
#[macro_use]
extern crate serde_derive;

use actix_web::{http, server, middleware, App, HttpResponse, HttpRequest, HttpContext, Form, Responder};
use actix_web::http::{StatusCode};
use actix::prelude::*;

extern crate env_logger;
mod eventsource;

pub struct SSEClient {
    topic: String,
}

pub struct SSEClientState {
    addr: Addr<eventsource::EventSource>,
}


impl Actor for SSEClient {
    type Context = HttpContext<Self, SSEClientState>;

    fn started(&mut self, ctx: &mut HttpContext<Self, SSEClientState>) {
        println!("SSEClient started");
        let addr = ctx.address();

        ctx.state().addr.send(eventsource::Connect {
            addr: addr.recipient(),
            topic: self.topic.clone(),
        })
            .into_actor(self)
            .then(|res, _act, ctx| {
                match res {
                    Ok(_) => (),
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                actix::fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        ctx.state().addr.send(eventsource::Disconnect {
            addr: ctx.address().recipient(),
        })
            .into_actor(self)
            .then(|_res, _act, _ctx| {
                actix::fut::ok(())
            })
            .wait(ctx);

        Running::Stop
    }
}

impl Handler<eventsource::SSEEvent> for SSEClient {
    type Result = ();

    fn handle(&mut self, msg: eventsource::SSEEvent, ctx: &mut HttpContext<Self, SSEClientState>) -> Self::Result {
        println!("Message {:?}", msg);
        if msg.topic == self.topic {
            ctx.write(format!("data: {}\n\n", msg.text));
        }
    }
}

fn follow_topic(req: &HttpRequest<SSEClientState>) -> HttpResponse {
    let me = SSEClient{ topic: req.match_info().get("topic").unwrap().to_string() };
    let ctx = HttpContext::create(req.clone(), me);

    let r = HttpResponse::build(StatusCode::OK)
        .content_type("text/event-stream")
        .content_encoding(http::header::ContentEncoding::Identity)
        .body(ctx);

    println!("sse: {:#?}", r.body());
    r
}

fn index(_req: &HttpRequest<SSEClientState>) -> impl Responder {
    let body = include_str!("index.html");
    HttpResponse::Ok().content_type("text/html").body(body)
}

#[derive(Deserialize)]
pub struct PostMessage {
    text: String,
}

fn post_to_topic((req, params): (HttpRequest<SSEClientState>, Form<PostMessage>)) -> HttpResponse {
    req.state().addr.do_send(eventsource::Publish {
        topic: req.match_info().get("topic").unwrap().to_string(),
        text: params.text.clone(),
    });
    let r = HttpResponse::build(StatusCode::OK)
        .content_type("text/plain")
        .body("posted");
    r
}

fn new_app(url_prefix: &str, addr: Addr<eventsource::EventSource>) -> App<SSEClientState> {

    let state = SSEClientState {
        addr: addr,
    };
    let prefix = format!("{}/{{topic}}", url_prefix);

    App::with_state(state)
        .middleware(middleware::Logger::default())
        .resource(&prefix, |r| {
            r.method(http::Method::POST).with(post_to_topic);
            r.method(http::Method::GET).f(follow_topic)
        })
        .resource("/", |r| r.method(http::Method::GET).f(index))
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let url_prefix = match std::env::var_os("SSEPLEX_URL_PREFIX") {
        Some(val) => val.to_str().unwrap().to_string(),
        None => "".to_string(),
    };

    let _sys = actix::System::new("sseplex");

    let sender = Arbiter::start(|_| eventsource::EventSource::default());

    if std::env::var("SSEPLEX_DUMMY_SENDER").is_ok() {
        sender.do_send(eventsource::StartDummySender{})
    }

    server::new(move || new_app(&url_prefix, sender.clone()))
        .bind("127.0.0.1:8080").unwrap()
        .run();
}
