use std::{sync::mpsc::TryRecvError, thread};

use godot::{engine::utilities::ceili, prelude::*};

use crate::{
    models::{
        driver_update::ManagerUpdate,
        manager::{ManagerCommand, ManagerCommandType, ManagerState},
        phone::Phone,
        prototype::Prototype,
        worker::{WorkerCommand, WorkerCommandType, WorkerUpdate},
    },
    worker::{chunk::Chunk, worker::Worker},
};

use super::{cell::Cell, library::Library3D};

pub struct Manager {
    state: ManagerState,

    // I/O
    phone: Phone<ManagerUpdate, ManagerCommand>,
    worker_phones: Vec<Phone<WorkerCommand, WorkerUpdate>>,

    // Map data
    library: Library3D<Cell>,
}

impl Manager {
    pub fn new(phone: Phone<ManagerUpdate, ManagerCommand>, map_size: Vector3i) -> Self {
        let chunk_size = Vector3i { x: 10, y: 6, z: 10 };
        let chunk_overlap = 2;

        let proto_data = Prototype::load();
        let cells = generate_cells(map_size, &proto_data);
        let library = Library3D::new(map_size, cells);

        Self {
            state: ManagerState::IDLE,
            phone,
            worker_phones: vec![],
            library,
        }
    }

    // CORE LOOP

    pub fn run(&mut self) {
        godot_print!("[M] Starting run");
        self.initialize();

        loop {
            match self.state {
                ManagerState::STOPPED => break,
                ManagerState::IDLE => self.wait_for_command(),
                ManagerState::PROCESSING => {
                    self.check_for_command(); // Check "upwards" for data from the main thread
                    self.check_for_messages(); // Check "downwards" for data from workers
                }
            }
        }
    }

    pub fn initialize(&mut self) {
        // self.map.initialize()
        // -> Post if changes

        let (mut phone_to_worker, phone_to_manager) = Phone::new_pair();

        let chunk = Chunk::new(
            Vector3i { x: 0, y: 0, z: 0 },
            Vector3i { x: 10, y: 6, z: 10 },
        );

        let (start, end) = chunk.bounds();
        let cells = self.library.check_out_range(start, end);
        match cells {
            Ok(cells) => {
                let _ =
                    phone_to_worker.send(WorkerCommand::new(WorkerCommandType::COLLAPSE, cells));
            }
            Err(e) => godot_print!("[M] Failed to check out range from library: {}", e),
        };

        self.worker_phones.push(phone_to_worker);

        let _worker_handle = thread::spawn(move || {
            let mut worker = Worker::new(phone_to_manager, chunk);
            worker.run()
        });
    }

    // INSTRUCTION FROM MAIN THREAD

    // Block on this thread until a new command is issued from the main thread
    fn wait_for_command(&mut self) {
        match self.phone.wait() {
            Ok(command) => self.on_command_received(command),
            Err(_) => self.set_state(ManagerState::STOPPED),
        }
    }

    // Check for a new command from the main thread, but do not block
    fn check_for_command(&mut self) {
        match self.phone.check() {
            Ok(command) => self.on_command_received(command),
            Err(e) => match e {
                TryRecvError::Empty => { /* No command */ }
                TryRecvError::Disconnected => self.set_state(ManagerState::STOPPED),
            },
        }
    }

    // Change state from command received
    // Logic here is very simple, but may want to be more complicated later
    fn on_command_received(&mut self, command: ManagerCommand) {
        godot_print!("[M] Command received: {:?}", command);
        match command.command {
            ManagerCommandType::NOOP => godot_print!("[M] noop!"),
            ManagerCommandType::START => self.set_state(ManagerState::IDLE),
            ManagerCommandType::PAUSE => self.set_state(ManagerState::PROCESSING),
            ManagerCommandType::STOP => self.set_state(ManagerState::STOPPED),
        }
    }

    // WORKER MANAGEMENT

    fn check_for_messages(&mut self) {
        for phone in &mut self.worker_phones {
            match phone.check() {
                Ok(msg) => godot_print!("[M] ok from worker: {:?}", msg),
                Err(e) => godot_print!("[M] err from worker: {}", e),
            }
        }
    }

    // HELPERS

    fn set_state(&mut self, new_state: ManagerState) {
        godot_print!("[M] State updated: {:?}", new_state);
        self.state = new_state;
        self.post_changes(ManagerUpdate::new_state(new_state));
    }

    fn post_changes(&mut self, update: ManagerUpdate) {
        if let Err(e) = self.phone.send(update) {
            godot_print!("[M] Failed to post changes! {}", e)
        }
    }
}

fn generate_cells(size: Vector3i, all_protos: &Vec<Prototype>) -> Vec<Cell> {
    let mut cells = vec![];
    for y in 0..size.y {
        for x in 0..size.x {
            for z in 0..size.z {
                let mut cell_protos = all_protos.clone();

                if x == 0 {
                    Prototype::retain_uncapped(&mut cell_protos, Vector3i::LEFT);
                } else if x == size.x - 1 {
                    Prototype::retain_uncapped(&mut cell_protos, Vector3i::RIGHT);
                }

                if y == 0 {
                    Prototype::retain_uncapped(&mut cell_protos, Vector3i::DOWN);
                } else {
                    Prototype::retain_not_constrained(&mut cell_protos, "BOT".into());
                    if y == size.y - 1 {
                        Prototype::retain_uncapped(&mut cell_protos, Vector3i::UP);
                    }
                }

                if z == 0 {
                    Prototype::retain_uncapped(&mut cell_protos, Vector3i::FORWARD);
                } else if z == size.z - 1 {
                    Prototype::retain_uncapped(&mut cell_protos, Vector3i::BACK);
                }

                let cell = Cell::new(Vector3i { x, y, z }, cell_protos);
                cells.push(cell);
            }
        }
    }
    cells
}

fn generate_chunks(size: Vector3i, chunk_size: Vector3i, chunk_overlap: i32) -> Vec<Chunk> {
    let num_x = ceili((size.x / (chunk_size.x - chunk_overlap)) as f64) as i32;
    let num_y = ceili((size.y / (chunk_size.y - chunk_overlap)) as f64) as i32;
    let num_z = ceili((size.z / (chunk_size.z - chunk_overlap)) as f64) as i32;
    let position_factor = chunk_size - Vector3i::ONE * chunk_overlap;

    let mut chunks = vec![];
    for y in 0..num_y {
        for x in 0..num_x {
            for z in 0..num_z {
                let position = position_factor * Vector3i { x, y, z };
                let new_chunk = Chunk::new(position, chunk_size);
                chunks.push(new_chunk);
            }
        }
    }

    chunks
}
