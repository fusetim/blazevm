use std::{collections::HashMap, ops::Deref};

use dumpster::{sync::Gc, Collectable};
use flagset::FlagSet;
use reader::base::{classfile::{self, ClassAccessFlags}, ClassFile};

use crate::{class_loader::{ClassLoader, ClassLoadingError}, class::{Class, ClassId, self}, constant_pool::ConstantPool};

/// Representation of the class manager.
///
/// It manages all the components linked or used to load classes at runtime.
#[derive(Debug)]
pub struct ClassManager {
    /// The class loader.
    pub class_loader: ClassLoader,

    /// The classes loaded by this class manager, indexed by their ID.
    pub classes_by_id: HashMap<ClassId, Gc<LoadedClass>>,

    /// The classes loaded by this class manager, indexed by their name.
    pub classes_by_name: HashMap<String, Gc<LoadedClass>>,

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

    pub fn get_or_resolve_class(&mut self, class_name: &str) -> Result<Gc<LoadedClass>, ClassLoadingError> {
        if let Some(class) = self.classes_by_name.get(class_name) {
            return Ok(class.clone());
        } else {
            let mut stack: Vec<String> = Vec::new();
            stack.push(class_name.to_string());
            while let Some(class_name) = stack.pop() {
                if let Some(class) = self.classes_by_name.get(&class_name) {
                    match class.deref() {
                        LoadedClass::Loaded(_) => (),
                        LoadedClass::Loading(loading) => {
                            // TODO: Check for circular dependencies.
                            // + Implement the final steps of class loading.
                        }
                    }
                } else {
                    let classfile = self.class_loader.load_classfile(&class_name)?;
                    let loaded_class = self.resolve_class(classfile)?;
                    self.classes_by_name.insert(class_name.clone(), loaded_class.clone());
                    self.classes_by_id.insert(loaded_class.id(), loaded_class.clone());
                    self.next_class_id = ClassId(self.next_class_id.0 + 1);
                    if let LoadedClass::Loading(loading) = loaded_class.deref() {
                        stack.push(class_name);
                        for dependency in loading.class_dependencies {
                            stack.push(dependency.clone());
                        }
                    }
                }
            }

            Ok(self.classes_by_name.get(class_name).unwrap().clone())
        }
    }

    pub fn resolve_class(&mut self, classfile: ClassFile) -> Result<Gc<LoadedClass>, ClassLoadingError> {
        let class_name = classfile.class_name()?;
        let class_id = self.next_class_id;
        let super_name = classfile.super_class_name()?;
        //let flags = classfile.access_flags();
        let interfaces = classfile.super_interfaces_names()?;
        let mut dependencies = Vec::new();
        if let Some(super_name) = super_name {
            dependencies.push(super_name.clone());
        }
        for interface in interfaces.iter() {
            dependencies.push(interface.clone());
        }
        Ok(Gc::new(LoadedClass::Loading(LoadingClass {
            class_id,
            class_name: class_name.to_string(),
            super_class: super_name.map(String::from),
            interfaces: interfaces.iter().map(|x| x.to_string()).collect(),
            // flags,
            // constant_pool: ConstantPool::new(classfile.constant_pool),
            class_dependencies: dependencies,
            //fields: Vec::new(),
            //methods: Vec::new(),
        })))
    }

}

#[derive(Debug, Clone, Collectable)]
pub enum LoadedClass {
    Loaded(Gc<Class>),
    Loading(LoadingClass),
}

impl LoadedClass {
    pub fn name(&self) -> &str {
        match self {
            LoadedClass::Loaded(class) => &class.name,
            LoadedClass::Loading(class) => &class.class_name,
        }
    }

    pub fn id(&self) -> ClassId {
        match self {
            LoadedClass::Loaded(class) => class.id,
            LoadedClass::Loading(class) => class.class_id,
        }
    }
}

#[derive(Debug, Clone, Collectable)]
pub struct LoadingClass {
    pub class_id: ClassId,
    pub class_name: String,
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,
    //pub flags: FlagSet<ClassAccessFlags>,
    // pub constant_pool: ConstantPool,
    pub class_dependencies: Vec<String>,
    //pub fields: Vec<class::Field>,
    //pub methods: Vec<class::Method>,
}
