use std::sync::mpsc;
use std::thread;

struct Worker {
    handle: Option<thread::JoinHandle<()>>,
    sender_to_worker: Option<mpsc::Sender<u32>>,
    receiver_from_worker: Option<mpsc::Receiver<u32>>,
    sender_to_main: Option<mpsc::Sender<u32>>,
}

impl Worker {
    fn new() -> Self {
        Worker {
            handle: None,
            sender_to_worker: None,
            receiver_from_worker: None,
            sender_to_main: None,
        }
    }

    fn start(&mut self) {
        let (sender_to_worker, receiver_from_main) = mpsc::channel();
        let (sender_to_main, receiver_from_worker) = mpsc::channel();

        self.sender_to_worker = Some(sender_to_worker);
        self.receiver_from_worker = Some(receiver_from_worker);
        self.sender_to_main = Some(sender_to_main);

        let sender_to_main = self.sender_to_main.take().unwrap();

        self.handle = Some(thread::spawn(move || {
            for received in receiver_from_main.iter() {
                // Simulate work by sleeping
                thread::sleep(std::time::Duration::from_secs(1));

                // Send data back to the main thread
                sender_to_main.send(received * 2).unwrap();
            }
        }));
    }

    fn send_data_to_worker(&mut self, value: u32) {
        if let Some(sender) = &self.sender_to_worker {
            sender.send(value).unwrap();
        }
    }

    fn receive_data_from_worker(&mut self) -> Option<u32> {
        if let Some(receiver) = &self.receiver_from_worker {
            receiver.recv().ok()
        } else {
            None
        }
    }

    fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

fn main() {
    let mut worker = Worker::new();

    worker.start();

    // Send data to the worker thread
    worker.send_data_to_worker(10);

    // Wait for the thread to complete its work
    worker.join();

    // Get the result from the worker thread
    if let Some(result) = worker.receive_data_from_worker() {
        godot_print!("Result from the worker thread: {}", result);
    }
}
