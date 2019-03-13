extern crate slack;
extern crate rand;
extern crate serde_json;

use slack::RtmClient;

use crate::shamebot::Shamebot;
mod shamebot;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let api_key = args[1].clone();
    let namespace = args[2].clone();
    let mut handler = Shamebot::new(&namespace);
    let r = RtmClient::login_and_run(&api_key, &mut handler);
    match r {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}