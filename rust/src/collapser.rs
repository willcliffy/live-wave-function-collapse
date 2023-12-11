use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use godot::prelude::*;

use crate::{
    chunk::Chunk,
    models::{
        collapser_action::{CollapserAction, CollapserActionType},
        collapser_state::CollapserState,
        driver_update::{DriverUpdate, SlotChange},
    },
};

pub struct LWFCCollapser {
    sender: Sender<DriverUpdate>,
    receiver: Receiver<CollapserAction>,

    state: CollapserState,
    chunks: Vec<Chunk>,
    current_chunk: usize,
}

impl LWFCCollapser {
    pub fn new(
        sender: Sender<DriverUpdate>,
        receiver: Receiver<CollapserAction>,
        _map_size: Vector3,
        _chunk_size: Vector3,
        _chunk_overlap: i32,
    ) -> Self {
        godot_print!("initialized");
        Self {
            sender,
            receiver,
            state: CollapserState::IDLE,
            chunks: vec![],
            current_chunk: 0,
        }
    }

    pub fn run(&mut self) {
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
                TryRecvError::Empty => self.collapse_next(),
                TryRecvError::Disconnected => {
                    godot_error!("Disconnected in thread (PROCESSING). Exiting.");
                    self.stop();
                }
            },
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

    fn collapse_next(&mut self) {
        let chunk = &mut self.chunks[self.current_chunk];
        let changed = chunk.collapse_next();
        match changed {
            Some(changes) => self.post_changes(changes),
            None => self.prepare_next_chunk(),
        }
    }

    fn prepare_next_chunk(&mut self) {
        self.current_chunk += 1;
        if self.current_chunk > self.chunks.len() {
            return self.idle();
        }

        let next_chunk = self.chunks.get(self.current_chunk);
        let mut overlapping: Vec<Vector3> = Vec::new();
        let mut neighboring: Vec<Vector3> = Vec::new();
        match next_chunk {
            None => {
                return godot_error!(
                    "Tried to get chunk that didn't exist! {} out of {} chunks",
                    self.current_chunk,
                    self.chunks.len()
                )
            }
            Some(next) => {
                for i in 0..self.current_chunk {
                    if let Some(other) = self.chunks.get(i) {
                        for slot in next.get_overlapping(other) {
                            overlapping.push(slot);
                        }

                        for slot in next.get_neighboring(other) {
                            neighboring.push(slot);
                        }
                    }
                }
            }
        }

        let next_chunk = self.chunks.get_mut(self.current_chunk);
        if let Some(next) = next_chunk {
            next.reset_slots(overlapping);
            next.propagate_from(neighboring);
            next.apply_custom_constraints();
        }
    }

    fn idle(&mut self) {
        self.state = CollapserState::IDLE;
        self.sender
            .send(DriverUpdate::new_state(self.state))
            .unwrap();
    }

    fn start(&mut self) {
        self.state = CollapserState::PROCESSING;
        self.sender
            .send(DriverUpdate::new_state(self.state))
            .unwrap();
    }

    fn stop(&mut self) {
        self.state = CollapserState::STOPPED;
        self.sender
            .send(DriverUpdate::new_state(self.state))
            .unwrap();
    }

    fn post_changes(&mut self, changes: Vec<SlotChange>) {
        self.sender
            .send(DriverUpdate::new_changes(changes))
            .unwrap();
    }
}
