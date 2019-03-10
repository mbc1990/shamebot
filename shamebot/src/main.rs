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
use std::io::BufReader;

const EYES: f32 = 0.95;
const EYES_NAME: f32 = 0.50;
const EYES_NAME_MESSAGE: f32 = 0.005;

pub fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}


struct Shamebot {
    deletes_by_user: HashMap<String, i32>,
}

impl Shamebot {
    fn new() -> Shamebot {
        let mut shamebot = Shamebot {
            deletes_by_user: HashMap::new(),
        };
        shamebot.load_counts();
        shamebot
    }
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
        Ok(())
    }

    fn load_counts(&mut self) -> std::io::Result<()> {
        if !path_exists(".counts.json") {
            self.deletes_by_user = HashMap::new();
            return Ok(());
        } else {
            let f = File::open(".counts.json").unwrap();
            let file = BufReader::new(&f);
            let mut l: String = "".to_string();
            for (num, line) in file.lines().enumerate() {
                l = line.unwrap();
            }
            let m: HashMap<String, Value> = serde_json::from_str(&l)?;
            for (key, value) in m {
                self.deletes_by_user.insert(key, value.as_i64().unwrap() as i32);
            }
            Ok(())
        }
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
                        let num_deleted = match self.deletes_by_user.get(&user) {
                            Some(count) => count.to_owned(),
                            None => 0
                        };

                        let mut rng = rand::thread_rng();
                        let dist = Uniform::from(0.0..1.0);
                        let roll = dist.sample(&mut rng);

                        if roll < EYES_NAME_MESSAGE {
                            let mut to_send = ":eyes: <@".to_string();
                            to_send.push_str(&user);
                            to_send.push_str(&"> ".to_string());
                            to_send.push_str(&text);
                            let _ = cli.sender().send_message(&channel, &to_send);
                        } else if roll  < EYES_NAME {
                            let mut to_send = ":eyes: <@".to_string();
                            to_send.push_str(&user);
                            to_send.push_str(&">".to_string());
                            let _ = cli.sender().send_message(&channel, &to_send);
                        } else if roll < EYES {
                            let to_send = ":eyes:";
                            let _ = cli.sender().send_message(&channel, &to_send);
                        }
                        self.deletes_by_user.insert(user, num_deleted + 1);
                        self.save_counts();
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
    let mut handler = Shamebot::new();
    let r = RtmClient::login_and_run(&api_key, &mut handler);
    match r {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}