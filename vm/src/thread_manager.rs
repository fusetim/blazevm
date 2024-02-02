use crate::thread::Thread;

#[derive(Debug, Clone)]
pub struct ThreadManager {
    pub threads: Vec<Thread>,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self { threads: vec![] }
    }

    pub fn create_thread<'a>(&'a mut self) -> &'a Thread {
        let thread = Thread::new();
        self.threads.push(thread);
        self.threads.last().unwrap()
    }

    pub fn get_thread(&self, index: usize) -> Option<&Thread> {
        self.threads.get(index)
    }

    pub fn get_thread_mut(&mut self, index: usize) -> Option<&mut Thread> {
        self.threads.get_mut(index)
    }

    pub fn stop_thread(&mut self, index: usize) {
        self.threads.remove(index);
    }
}
