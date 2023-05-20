use std::{thread, time::Duration};

use actix::{prelude::*, Actor, AsyncContext, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use faker::{commands::Command, twitch_api::TwitchUser};

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

            // Iterate over all possible messages at once, rather than waiting a second to send the next one
            while let Some((cmd, username)) = faker::MESSAGES.lock().pop_front() {
                println!("Found a message");
                // Skip any comments or empty lines

                let delay = cmd.get_delay();

                debug!("Sleeping for {} milliseconds", delay.as_millis());

                thread::sleep(delay);

                debug!("Sending message");

                debug!("{:?}", cmd);

                match cmd {
                    Command::Send {
                        ref message,
                        count,
                        delay: _,
                    } => {
                        for _ in 0..count {
                            let user = {
                                if username == "random" {
                                    TwitchUser::random()
                                } else {
                                    TwitchUser::from_username(&username)
                                }
                            };

                            let parsed = user.send_message(message);
                            ctx.text(parsed);

                            debug!("Sleeping for {} milliseconds", delay.as_millis());

                            thread::sleep(delay);
                        }
                    }
                    Command::Sleep { delay: _ } => thread::sleep(delay),
                }

                debug!("Message sent");
            }
            println!("Done sending messages");
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
