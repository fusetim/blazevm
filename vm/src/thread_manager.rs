use crate::{
    class::{Class, ClassId},
    thread::{Frame, Slot, Thread},
};

pub type ThreadId = usize;

#[derive(Debug, Clone)]
pub struct ThreadManager {
    pub threads: Vec<Thread>,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self { threads: vec![] }
    }

    pub fn create_thread<'a>(
        &'a mut self,
        class: &ClassId,
        method: usize,
        max_locals: usize,
        args: Vec<Slot>,
    ) -> ThreadId {
        let mut thread = Thread::new();

        thread.push_frame(Frame::new(class.clone(), method, max_locals));
        let mut pos = 0;
        for arg in args {
            if arg.size() > 1 {
                pos += 1;
            }
            *thread
                .current_frame_mut()
                .unwrap()
                .get_local_variable_mut(pos)
                .unwrap() = arg;
            pos += 1;
        }
        self.threads.push(thread);
        return self.threads.len() - 1;
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
