extern crate slack;
use slack::{Event, RtmClient, Message};

struct Shamebot;

#[allow(unused_variables)]
impl slack::EventHandler for Shamebot {
    fn on_event(&mut self, cli: &RtmClient, event: Event) {
        match event {
            Event::Message(msg) => {
                match *msg {
                    Message::MessageDeleted(del_message) => {
                        let channel = del_message.channel.unwrap();
                        let prev_message = del_message.previous_message.unwrap();
                        let user = prev_message.user.unwrap();
                        let text = prev_message.text.unwrap();
                        println!("msg deleted: {:?}, {:?}, {:?})", channel, user, text);
                        let mut to_send = ":eyes: <@".to_string();
                        to_send.push_str(&user);
                        to_send.push_str(&">".to_string());
                        let _ = cli.sender().send_message(&channel, &to_send);
                    }
                    _ => {}
                }
            },
            _ => {}
        }
    }
    fn on_close(&mut self, cli: &RtmClient) {
        println!("Connection closed");
    }

    fn on_connect(&mut self, cli: &RtmClient) {
        println!("Shamebot connected");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let api_key = match args.len() {
        0 | 1 => panic!("No api key"),
        x => args[x - 1].clone(),
    };
    let mut handler = Shamebot;
    let r = RtmClient::login_and_run(&api_key, &mut handler);
    match r {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}