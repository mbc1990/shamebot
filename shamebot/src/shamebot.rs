use slack::{Event, RtmClient, Message};
use rand::distributions::{Distribution, Uniform};
use std::collections::HashMap;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::fs;
use std::io::BufReader;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::prelude::SliceRandom;


// TODO: Move to main and pass to constructor
const EYES: f32 = 0.33;
const EYES_NAME: f32 = 0.25;
const EYES_NAME_MESSAGE: f32 = 0.001;
const HESITATION_TIMEOUT_SECS: u64 = 7;
const HESITATION_COOLDOWN_SECS: u64 = 10;


pub fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

pub struct Shamebot<'a> {
    deletes_by_user: HashMap<String, i32>,

    // channel+user -> When the current typing series started
    typing_started: HashMap<String, u64>,
    // channel+user -> The last time the user typed (for cooldown)
    last_typing: HashMap<String, u64>,
    namespace: &'a String,
    hesitation_messages: Vec<&'a str>,
}

impl<'a> Shamebot<'a> {
    pub fn new(namespace: &String) -> Shamebot {
        let messages = vec![
            "spit it out!",
            "_checks watch_",
            "this better be worth it",
            "https://www.typingclub.com/",
            "if you have something to say, now's the time",
            "just text me when you're finished",
            "is this a slack message or are you transcribing infinite jest",
            "tippity tappity your typing isn't bappity",
            "are your fingers feeling okay?"
        ];
        let mut shamebot = Shamebot {
            deletes_by_user: HashMap::new(),
            typing_started: HashMap::new(),
            last_typing: HashMap::new(),
            namespace,
            hesitation_messages: messages,
        };
        shamebot.load_counts().unwrap();
        shamebot
    }
    fn save_counts(&self) -> std::io::Result<()> {
        let mut counts_path = ".counts.".to_string();
        counts_path.push_str(self.namespace);
        counts_path.push_str(&".json".to_string());
        if !path_exists(&counts_path) {
            File::create(&counts_path)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&counts_path)
            .unwrap();
        let ser = serde_json::to_string(&self.deletes_by_user)?;

        if let Err(e) = writeln!(file, "{}", ser) {
            eprintln!("Couldn't write to file: {}", e);
        }
        Ok(())
    }

    fn load_counts(&mut self) -> std::io::Result<()> {
        let mut counts_path = ".counts.".to_string();
        counts_path.push_str(self.namespace);
        counts_path.push_str(&".json".to_string());
        if !path_exists(&counts_path) {
            self.deletes_by_user = HashMap::new();
            return Ok(());
        } else {
            let f = File::open(counts_path).unwrap();
            let file = BufReader::new(&f);
            let mut l: String = "".to_string();
            for (_, line) in file.lines().enumerate() {
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
impl<'a> slack::EventHandler for Shamebot<'a> {
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
                        self.save_counts().unwrap();
                    },
                    Message::Standard(msg) => {
                        let channel = msg.channel.unwrap();
                        let user = msg.user.unwrap();
                        let mut key = channel.to_string();
                        key.push_str(&"-");
                        key.push_str(&user);
                        self.typing_started.remove(&key);
                        self.last_typing.remove(&key);
                    },
                    // TODO: On message clear typing_started entry for channel+user
                    _ => {}
                }
            },
            Event::UserTyping {channel, user} => {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                let mut to_remove = Vec::new();

                // First, check everything for cooldown and remove it if it's expired
                for (k, v) in self.last_typing.iter() {
                    if now - v > HESITATION_COOLDOWN_SECS {
                        to_remove.push(k.clone());
                    }
                }

                for expired in to_remove.iter() {
                    self.last_typing.remove(&expired.to_string());
                    self.typing_started.remove(&expired.to_string());
                }

                let mut key = channel.to_string();
                key.push_str(&"-");
                key.push_str(&user);
                if self.typing_started.contains_key(&key) {
                    let typing_started_event = self.typing_started.get(&key).unwrap();
                    if now - typing_started_event > HESITATION_TIMEOUT_SECS {
                        let mut to_send = "<@".to_string();
                        to_send.push_str(&user);
                        to_send.push_str(&"> ".to_string());
                        let mut rng = rand::thread_rng();
                        // let msg = rng.choose(&self.hesitation_messages).unwrap();
                        let msg = self.hesitation_messages.choose(&mut rng).unwrap();
                        to_send.push_str(msg);
                        let _ = cli.sender().send_message(&channel, &to_send);
                        self.typing_started.remove(&key);
                        self.last_typing.remove(&key);
                    }
                } else {
                    self.typing_started.insert((&key).to_string(), now);
                }

                // Update last typing to now
                self.last_typing.insert((&key).to_string(), now);
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
