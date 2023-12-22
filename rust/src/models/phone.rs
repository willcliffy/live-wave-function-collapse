use std::sync::mpsc::{channel, Receiver, RecvError, SendError, Sender, TryRecvError};

pub struct Phone<S, R> {
    sender: Sender<S>,
    receiver: Receiver<R>,
}

impl<S, R> Phone<S, R> {
    fn new(sender: Sender<S>, receiver: Receiver<R>) -> Self {
        Self { sender, receiver }
    }

    pub fn new_pair() -> (Phone<S, R>, Phone<R, S>) {
        let (s1, r1) = channel::<S>();
        let (s2, r2) = channel::<R>();
        (Phone::new(s1, r2), Phone::new(s2, r1))
    }

    pub fn wait(&mut self) -> Result<R, RecvError> {
        self.receiver.recv()
    }

    pub fn check(&mut self) -> Result<R, TryRecvError> {
        self.receiver.try_recv()
    }

    pub fn send(&mut self, message: S) -> Result<(), SendError<S>> {
        self.sender.send(message)
    }
}
