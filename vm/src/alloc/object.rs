use std::sync::RwLock;

use dumpster::{sync::Gc, Collectable};
use reader::{
    base::{classfile::FieldAccessFlags, ClassFile},
    descriptor,
};

use crate::{
    class::ClassId,
    class_loader::ClassLoadingError,
    class_manager::{ClassManager, LoadedClass},
    constant_pool::ConstantPoolError,
    slot::Slot,
};

pub type ObjectRef = Gc<Object>;

#[derive(Debug, Collectable)]
pub struct Object {
    class_id: ClassId,
    fields: RwLock<Vec<Slot>>,
    // A better solution would have been to use Once but unfortunately it does not
    // implement Collectable.
    initialized: RwLock<ObjectInitState>,
}

impl Object {
    /// Create a new object
    ///
    /// Note: The fields should be initialized to their default value, moreover
    /// static fields can be replaced by a Tombsone slot.
    pub fn new(class_id: ClassId, fields: Vec<Slot>) -> Self {
        Self {
            class_id,
            fields: RwLock::new(fields),
            initialized: RwLock::new(ObjectInitState::Uninitialized),
        }
    }

    /// Create a new object and load the class if necessary
    ///
    /// This method will request the class to be loaded if it is not already
    /// loaded, then it will create a new object with the default values for the
    /// non-static fields.
    pub fn new_with_classmanager(
        cm: &mut ClassManager,
        class_id: ClassId,
    ) -> Result<Self, ClassLoadingError> {
        cm.request_class_load(class_id)?;
        let Some(LoadedClass::Loaded(class)) = cm.get_class_by_id(class_id) else {
            log::debug!("Class not loaded: {:?}", class_id);
            return Err(ClassLoadingError::Unknown);
        };
        let mut fields = vec![];
        for f in class.fields.iter() {
            if f.is_static() {
                fields.push(Slot::Tombstone);
            } else {
                fields.push(Slot::default_for(f.descriptor.field_type()));
            }
        }

        Ok(Self::new(class_id, fields))
    }

    pub(crate) fn new_with_classfile(
        class_id: ClassId,
        classfile: &ClassFile,
    ) -> Result<Self, ClassLoadingError> {
        let mut fields = vec![];
        for f in classfile.fields().iter() {
            if f.access_flags.contains(FieldAccessFlags::Static) {
                fields.push(Slot::Tombstone);
            } else {
                let Some(descriptor) = classfile
                    .constant_pool()
                    .get_utf8_string(f.descriptor_index as usize)
                else {
                    log::error!("alloc::object::new_with_classfile - Failed to get field descriptor from constant pool");
                    return Err(ClassLoadingError::ConstantPoolLoadingError {
                        source: ConstantPoolError::InvalidUtf8StringReference {
                            index: f.descriptor_index as usize,
                        },
                    });
                };
                let desc = descriptor::parse_field_descriptor(&descriptor.to_string()).map_err(|err| {
                    log::error!("alloc::object::new_with_classfile - Failed to parse field descriptor: {}", &err);
                    ClassLoadingError::BadDescriptor { source: err }
                })?;
                fields.push(Slot::default_for(desc.field_type()));
            }
        }

        Ok(Self::new(class_id, fields))
    }

    /// Get the class id of the object
    pub fn class_id(&self) -> &ClassId {
        &self.class_id
    }

    /// Check if the object has been initialized
    pub fn is_initialized(&self) -> bool {
        *self
            .initialized
            .read()
            .expect("rwlock has been poisoned, cannot read initialized flag")
            == ObjectInitState::Initialized
    }

    /// Check if the object is currently being initialized
    pub fn is_initializing(&self) -> bool {
        *self
            .initialized
            .read()
            .expect("rwlock has been poisoned, cannot read initialized flag")
            == ObjectInitState::Initializing
    }

    /// Set the initialization state of the object
    pub fn set_init_state(&self, state: ObjectInitState) {
        *self
            .initialized
            .write()
            .expect("rwlock has been poisoned, cannot set initialized flag") = state;
    }

    /// Get the value at the given index
    pub fn get_field(&self, index: usize) -> Option<Slot> {
        self.fields
            .read()
            .expect("rwlock has been poisoned, cannot get field of object")
            .get(index)
            .cloned()
    }

    /// Set the value at the given index
    pub fn set_field(&self, index: usize, value: Slot) {
        self.fields
            .write()
            .expect("rwlock has been poisoned, cannot set field of object")[index] = value;
    }
}

#[derive(Debug, Collectable, Clone, Copy, PartialEq, Eq)]
pub enum ObjectInitState {
    Uninitialized,
    Initializing,
    Initialized,
    Failed,
}
