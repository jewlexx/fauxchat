use std::{thread, time::Duration};

use actix::{prelude::*, Actor, AsyncContext, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use rand::Rng;

use faker::commands::Command;

#[allow(clippy::unused_async, clippy::needless_pass_by_value)]
pub async fn handle_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(FakeIrc { addrs: vec![] }, &req, stream);
    println!("{resp:?}");
    resp
}

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[allow(clippy::module_name_repetitions)]
pub struct FakeIrc {
    pub addrs: Vec<Recipient<Message>>,
}

// TODO: Add Heartbeats

impl Actor for FakeIrc {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Creating message sender interval");

        ctx.run_interval(Duration::from_secs(1), move |_, ctx| {
            debug!("Sending message");

            let mut rng = rand::thread_rng();

            let msg = faker::MESSAGES.lock().pop_front();

            if let Some((msg, user)) = msg {
                // Skip any comments or empty lines
                if msg.starts_with('#') || msg.is_empty() {
                    return;
                }

                let millis: u64 = rng.gen_range(50..1500);

                debug!("Sleeping for {} milliseconds", millis);

                thread::sleep(Duration::from_millis(millis));

                debug!("Sending message");

                debug!("{}", msg);

                let parsed = Command::try_from(msg).unwrap();

                match parsed {
                    Command::Send(ref message, count) => {
                        for _ in 0..count {
                            let parsed = user.send_message(message);
                            ctx.text(parsed);

                            let millis: u64 = rng.gen_range(50..1500);

                            debug!("Sleeping for {} milliseconds", millis);

                            thread::sleep(Duration::from_millis(millis));
                        }
                    }
                    Command::Sleep(millis) => thread::sleep(Duration::from_millis(millis)),
                }

                debug!("Message sent");
            } else {
                debug!("No message to print");
            }
        });
    }
}

/// Handler for `ws::Message` message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for FakeIrc {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                info!("Received: {}", text);
            }
            _ => (),
        }
    }
}