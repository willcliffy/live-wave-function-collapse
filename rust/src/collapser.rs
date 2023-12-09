// use std::thread::{self, JoinHandle};

use std::collections::VecDeque;
use std::sync::mpsc::{self, Sender};
use std::thread;

use godot::prelude::*;

use crate::models::{self, LWFCCollapserActionType};

// const AUTOCOLLAPSE_SPEED: i32 = 5;

#[derive(GodotClass)]
pub struct LWFCCollapser {
    pub idle: bool,
    pub queued_actions: VecDeque<models::LWFCCollapserAction>,
    pub sender: Option<Sender<models::LWFCCollapserAction>>,
}

#[godot_api]
impl LWFCCollapser {
    fn initialize(mut self) {
        let (tx, rx) = mpsc::channel::<models::LWFCCollapserAction>();
        let initialize_action = models::LWFCCollapserAction {
            action_type: LWFCCollapserActionType::INITIALIZE,
            payload: None,
        };

        println!("{}", tx.send(initialize_action).unwrap_err());

        thread::spawn(move || loop {
            self.idle = true;
            let received = rx.recv();
            self.idle = false;
            let msg = match received {
                Ok(action) => action,
                Err(err) => {
                    println!("Rcv err: {}", err);
                    continue;
                }
            };

            match msg.action_type {
                LWFCCollapserActionType::NOOP => continue,
                LWFCCollapserActionType::INITIALIZE => {}
                LWFCCollapserActionType::START => {}
                LWFCCollapserActionType::STOP => {}
            }
        });
    }

    fn start(self) {}

    fn stop(self) {}
}
