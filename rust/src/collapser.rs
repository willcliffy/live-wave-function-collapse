use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use godot::prelude::*;

use crate::{
    map::Map,
    models::{
        collapser_action::{CollapserAction, CollapserActionType},
        collapser_state::CollapserState,
        driver_update::DriverUpdate,
    },
};

pub struct LWFCCollapser {
    state: CollapserState,

    sender: Sender<DriverUpdate>,
    receiver: Receiver<CollapserAction>,

    map: Map,
}

impl LWFCCollapser {
    pub fn new(
        sender: Sender<DriverUpdate>,
        receiver: Receiver<CollapserAction>,
        map_size: Vector3i,
        chunk_size: Vector3i,
        chunk_overlap: i32,
    ) -> Self {
        let state = CollapserState::IDLE;
        let map = Map::new(map_size, chunk_size, chunk_overlap);
        Self {
            state,
            sender,
            receiver,
            map,
        }
    }

    pub fn run(&mut self) {
        godot_print!("Starting run in thread.");
        if let Some(update) = self.map.initialize() {
            self.post_changes(update);
        }

        loop {
            if self.state == CollapserState::STOPPED {
                break;
            }

            if self.state == CollapserState::IDLE {
                self.wait_for_message();
            }

            if self.state == CollapserState::PROCESSING {
                self.check_for_message();
            }
        }
    }

    fn wait_for_message(&mut self) {
        godot_print!("Waiting for message in thread");
        match self.receiver.recv() {
            Ok(action) => self.on_message_received(action),
            Err(e) => {
                godot_error!("Disconnected in thread (IDLE). Exiting. {}", e);
                self.stop();
            }
        }
    }

    fn check_for_message(&mut self) {
        match self.receiver.try_recv() {
            Ok(action) => self.on_message_received(action),
            Err(e) => match e {
                TryRecvError::Empty => {
                    self.collapse_next();
                }
                TryRecvError::Disconnected => {
                    godot_error!("Disconnected in thread (PROCESSING). Exiting.");
                    self.stop()
                }
            },
        }
    }

    fn collapse_next(&mut self) {
        let update = self.map.collapse_next();
        if let Some(update) = update {
            if let Some(state) = update.new_state {
                // TODO - apply the state update here if we have one.
                // the collapser should have responsibility to stop itself
                // plus the driver doesnt currently do anything with status updates.
                if state == CollapserState::STOPPED {
                    return self.stop();
                }
            }

            // TODO - this leads to uneven updates from the thread.
            // Sometimes you'll post one change per tick and the main thread never catches up.
            // Sometimes you'll post one large collapse and it will try to process the entire thing in one tick
            self.post_changes(update)
        }
    }

    fn on_message_received(&mut self, action: CollapserAction) {
        godot_print!("Message received in thread: {:?}", action);
        match action.action_type {
            CollapserActionType::NOOP => godot_print!("noop!"),
            CollapserActionType::START => self.start(),
            CollapserActionType::PAUSE => self.idle(),
            CollapserActionType::STOP => self.stop(),
        }
    }

    fn idle(&mut self) {
        self.state = CollapserState::IDLE;
        self.post_changes(DriverUpdate::new_state(self.state));
    }

    fn start(&mut self) {
        self.state = CollapserState::PROCESSING;
        self.post_changes(DriverUpdate::new_state(self.state));
    }

    fn stop(&mut self) {
        self.state = CollapserState::STOPPED;
        self.post_changes(DriverUpdate::new_state(self.state));
    }

    fn post_changes(&mut self, update: DriverUpdate) {
        self.sender.send(update).unwrap();
    }
}
