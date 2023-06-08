use std::thread;

use actix::{prelude::*, Actor, AsyncContext, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use commands::Command;
use crossbeam::channel::Receiver;
use parking_lot::Mutex;
use twitch_api::TwitchUser;

#[allow(clippy::unused_async, clippy::needless_pass_by_value)]
pub async fn handle_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(FakeIrc, &req, stream);
    dbg!(resp)
}

pub type SingleCommand = (Command, String);

pub static RECIPIENTS: Mutex<Vec<Recipient<Message>>> = Mutex::new(Vec::new());

pub fn send_messages(receiver: &Receiver<SingleCommand>) {
    use std::fs::OpenOptions;

    let conns = RECIPIENTS.lock().len();
    debug!("{conns} connections");
    debug!("Sending message");

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("current.messages")
        .unwrap();

    // While loop will exit once connection is closed
    while let Ok((cmd, username)) = receiver.recv() {
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

                    for conn in RECIPIENTS.lock().iter() {
                        conn.do_send(Message(parsed.clone()));
                    }

                    debug!("Sleeping for {} milliseconds", delay.as_millis());

                    thread::sleep(delay);
                }
            }
            Command::Sleep { delay: _ } => {
                thread::sleep(delay);
            }
        }

        debug!("Message sent");
    }
    debug!("Done sending messages");
}

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[allow(clippy::module_name_repetitions)]
pub struct FakeIrc;

// TODO: Add Heartbeats

impl actix::Handler<Message> for FakeIrc {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

impl Actor for FakeIrc {
    type Context = ws::WebsocketContext<Self>;

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let mut recipients = RECIPIENTS.lock();
        let index = recipients
            .iter()
            .position(|addr| addr == &ctx.address().recipient());

        if let Some(index) = index {
            recipients.remove(index);
        }
    }

    fn started(&mut self, ctx: &mut Self::Context) {
        RECIPIENTS.lock().push(ctx.address().recipient::<Message>());

        info!("Creating message sender interval");

        // ctx.run_interval(Duration::from_secs(1), move |irc, ctx| {
        //     let conns = irc.recipients.len();
        //     debug!("{conns} connections");
        //     debug!("Sending message");

        //     // Should allow us to grab unsent messages as many times as we have connections, meaning that every connection will get the messages
        //     debug!("Getting unsent messages");
        //     let messages = load_unsent(conns);
        //     debug!("Got unsent messages");

        //     // Iterate over all possible messages at once, rather than waiting a second to send the next one
        //     for (cmd, username) in messages {
        //         println!("Found a message");
        //         // Skip any comments or empty lines

        //         let delay = cmd.get_delay();

        //         debug!("Sleeping for {} milliseconds", delay.as_millis());

        //         thread::sleep(delay);

        //         debug!("Sending message");

        //         debug!("{:?}", cmd);

        //         match cmd {
        //             Command::Send {
        //                 ref message,
        //                 count,
        //                 delay: _,
        //             } => {
        //                 for _ in 0..count {
        //                     let user = {
        //                         if username == "random" {
        //                             TwitchUser::random()
        //                         } else {
        //                             TwitchUser::from_username(&username)
        //                         }
        //                     };

        //                     let parsed = user.send_message(message);
        //                     ctx.text(parsed);

        //                     debug!("Sleeping for {} milliseconds", delay.as_millis());

        //                     thread::sleep(delay);
        //                 }
        //             }
        //             Command::Sleep { delay: _ } => thread::sleep(delay),
        //         }

        //         debug!("Message sent");
        //     }
        //     debug!("Done sending messages");
        // });
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
