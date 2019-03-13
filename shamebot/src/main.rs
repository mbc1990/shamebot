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