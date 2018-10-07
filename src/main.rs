extern crate actix_web;
extern crate actix;
extern crate tokio_timer;
use actix_web::{http, server, middleware, App, HttpResponse, HttpRequest, HttpContext};
use actix_web::http::{StatusCode};
use actix::prelude::*;

mod eventsource;

pub struct SSEClient {
    id: usize,
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
        })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                println!("my id {}", act.id);
                actix::fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        println!("XXXXXXX Stopping {}", self.id);
        ctx.state().addr.send(eventsource::Disconnect {
            id: self.id,
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
        println!("Message {} {:?}", self.id, msg);
        ctx.write(format!("data: {}\n\n", msg.text));
    }
}

fn sse(req: &HttpRequest<SSEClientState>) -> HttpResponse {
    let me = SSEClient{ id : 0,};
    let ctx = HttpContext::create(req.clone(), me);

    let r = HttpResponse::build(StatusCode::OK)
        .content_type("text/event-stream")
        .body(ctx);

    println!("sse: {:#?}", r.body());
    r
}

fn new_app(addr: Addr<eventsource::EventSource>) -> App<SSEClientState> {

    let state = SSEClientState {
        addr: addr,
    };

    App::with_state(state)
        .middleware(middleware::Logger::default())
        .resource("/{topic}", |r| r.method(http::Method::GET).f(sse))
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let _sys = actix::System::new("sseplex");

    let sender = Arbiter::start(|_| eventsource::EventSource::default());

    server::new(move || new_app(sender.clone()))
        .bind("127.0.0.1:8080").unwrap()
        .run();
}
