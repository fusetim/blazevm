use std::collections::HashMap;

use dumpster::sync::Gc;
use reader::base::{classfile, ClassFile};

use crate::{class_loader::{ClassLoader, ClassLoadingError}, class::{Class, ClassId, self}};

/// Representation of the class manager.
///
/// It manages all the components linked or used to load classes at runtime.
#[derive(Debug)]
pub struct ClassManager {
    /// The class loader.
    pub class_loader: ClassLoader,

    /// The classes loaded by this class manager, indexed by their ID.
    pub classes_by_id: HashMap<ClassId, Gc<Class>>,

    /// The classes loaded by this class manager, indexed by their name.
    pub classes_by_name: HashMap<String, Gc<Class>>,

    /// The next class ID to use.
    next_class_id: ClassId,
}

impl ClassManager {
    pub fn new(class_loader: ClassLoader) -> Self {
        Self {
            class_loader,
            classes_by_id: HashMap::new(),
            classes_by_name: HashMap::new(),
            next_class_id: ClassId(0),
        }
    }

    pub fn get_or_resolve_class(&mut self, class_name: &str) -> Result<Gc<Class>, ClassLoadingError> {
        if let Some(class) = self.classes_by_name.get(class_name) {
            return Ok(class.clone());
        } else {
            let classfile = self.class_loader.load_classfile(class_name)?;
            let class = self.initialize_class(classfile)?;
            Ok(class)
        }
    }

    pub fn initialize_class(&mut self, classfile: ClassFile) -> Result<Gc<Class>, ClassLoadingError> {
        unimplemented!("ClassManager::initialize_class unimplemented!");
    }
}