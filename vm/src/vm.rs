use crate::{
    class_loader::ClassLoader, class_manager::ClassManager, thread_manager::ThreadManager,
};

#[derive(Debug)]
pub struct Vm {
    class_manager: ClassManager,

    thread_manager: ThreadManager,
}

impl Vm {
    pub fn new(cl: ClassLoader) -> Self {
        Self {
            class_manager: ClassManager::new(cl),
            thread_manager: ThreadManager::new(),
        }
    }

    pub fn class_manager(&self) -> &ClassManager {
        &self.class_manager
    }

    pub fn class_manager_mut(&mut self) -> &mut ClassManager {
        &mut self.class_manager
    }

    pub fn thread_manager(&self) -> &ThreadManager {
        &self.thread_manager
    }

    pub fn thread_manager_mut(&mut self) -> &mut ThreadManager {
        &mut self.thread_manager
    }
}
