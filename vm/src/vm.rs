use crate::{
    class::ClassId,
    class_loader::ClassLoader,
    class_manager::{ClassManager, LoadedClass},
    thread::{ExecutionError, Slot},
    thread_manager::ThreadManager,
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

    pub fn create_thread(&mut self, class_id: &ClassId, method: usize, args: Vec<Slot>) -> usize {
        let Some(LoadedClass::Loaded(class)) = self.class_manager.get_class_by_id(class_id.clone())
        else {
            panic!("Class not loaded: {:?}", class_id);
        };
        let m = class.get_method_by_index(method).unwrap();
        let code = m.get_code().expect(
            "Code attribute not found, probably a native method, unsupported as thread entry point",
        );
        let max_locals = code.max_locals as usize;

        self.thread_manager
            .create_thread(&class_id, method, max_locals, args)
    }

    pub fn execute_thread(&mut self, thread_id: usize) -> Result<(), ExecutionError> {
        let thread = self.thread_manager.get_thread_mut(thread_id).unwrap();
        let x = thread.execute(&mut self.class_manager);
        log::debug!("Classes loaded: {}", self.class_manager.classes_by_id.len());
        log::debug!("Classes by names: {:?}", &self.class_manager.name_map);
        x
    }
}
