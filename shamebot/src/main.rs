extern crate slack;
extern crate rand;
extern crate serde_json;


use slack::{Event, RtmClient, Message};
use rand::Rng;
use rand::distributions::{Distribution, Uniform};
use std::collections::HashMap;
use serde_json::{Result, Value};
use std::fs::File;
use std::io::prelude::*;
use std::error::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::fs;
use std::time::{Duration, SystemTime};


pub fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}


struct Shamebot {
    deletes_by_user: HashMap<String, i32>,
}

impl Shamebot {
    fn save_counts(&self) -> std::io::Result<()> {
        if !path_exists(".counts.json") {
            File::create(".counts.json")?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(".counts.json")
            .unwrap();
        let ser = serde_json::to_string(&self.deletes_by_user)?;

        if let Err(e) = writeln!(file, "{}", ser) {
            eprintln!("Couldn't write to file: {}", e);
        }
        println!("{:?}", ser);
        Ok(())
    }

    // TODO
    fn load_counts(&mut self) -> std::io::Result<()> {
        
        let ser = serde_json::to_string(&self.deletes_by_user)?;
        println!("{:?}", ser);
        Ok(())
    }
}


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

                        // TODO: Use into API
                        let num_deleted = match self.deletes_by_user.get(&user) {
                            Some(count) => count.to_owned(),
                            None => 0
                        };
                        self.deletes_by_user.insert(user, num_deleted + 1);
                        self.save_counts();

                        println!("Num deleted: {:?}", num_deleted);

                        let mut rng = rand::thread_rng();
                        let dist = Uniform::from(0.0..1.0);
                        let roll = dist.sample(&mut rng);
                        if roll < 0.25 {
                            let to_send = ":eyes:";
                            let _ = cli.sender().send_message(&channel, &to_send);
                        }
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
    let mut handler = Shamebot
        {
            deletes_by_user: HashMap::new(),
        };
    let r = RtmClient::login_and_run(&api_key, &mut handler);
    match r {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}