use std::{cell::OnceCell, collections::HashMap};

use flagset::FlagSet;
use reader::{
    base::{
        classfile::ClassAccessFlags,
        constant_pool::{ConstantPoolEntry, ConstantPoolInfo},
        ClassFile,
    },
    descriptor::{self, MethodDescriptor},
};

use crate::{
    class::{self, Class, ClassId},
    class_loader::{ClassLoader, ClassLoadingError, DerivingError},
    constant_pool::{ConstantPool, ConstantPoolError},
    thread::{ExecutionError, Frame, Thread},
};

const CLINIT_DESCRIPTOR: MethodDescriptor = MethodDescriptor {
    return_type: None,
    parameters: vec![],
};

/// Representation of the class manager.
///
/// It manages all the components linked or used to load classes at runtime.
#[derive(Debug)]
pub struct ClassManager {
    /// The class loader.
    pub class_loader: ClassLoader,

    /// The classes loaded by this class manager, indexed by their ID.
    pub classes_by_id: HashMap<ClassId, LoadedClass>,

    /// The classes loaded by this class manager, indexed by their name.
    pub classes_by_name: HashMap<String, LoadedClass>,

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

    fn execute_class_init(
        &mut self,
        thread: &mut Thread,
        class_id: &ClassId,
    ) -> Result<(), ExecutionError> {
        thread.reset();
        let clid = {
            let Some(LoadedClass::Loaded(class)) = self.classes_by_id.get(class_id) else {
                return Err(ExecutionError::ClassNotLoaded);
            };
            class.index_of_method("<clinit>", &CLINIT_DESCRIPTOR)
        };
        if let Some(clid) = clid {
            let frame = Frame::new(*class_id, clid, 0);
            thread.push_frame(frame);
            thread.execute(self)?;
        }
        let Some(LoadedClass::Loaded(class)) = self.classes_by_id.get_mut(class_id) else {
            return Err(ExecutionError::ClassNotLoaded);
        };
        class.initialized = OnceCell::new();
        class.initialized.set(true).unwrap();
        Ok(())
    }

    pub fn get_class_by_id(&self, id: ClassId) -> Option<&LoadedClass> {
        self.classes_by_id.get(&id)
    }

    pub fn get_mut_class_by_id(&mut self, id: ClassId) -> Option<&mut LoadedClass> {
        self.classes_by_id.get_mut(&id)
    }

    pub fn get_or_resolve_class(
        &mut self,
        class_name: &str,
    ) -> Result<&LoadedClass, ClassLoadingError> {
        if !self.classes_by_name.contains_key(class_name) {
            let mut init_thread = Thread::new();
            let mut stack: Vec<String> = Vec::new();
            stack.push(class_name.to_string());
            while let Some(class_name) = stack.pop() {
                log::debug!("Resolving class: {}", &class_name);
                if let Some(class) = self.classes_by_name.get(&class_name) {
                    let class = class.clone();
                    match class {
                        LoadedClass::Loaded(_) => (),
                        LoadedClass::Loading(loading) => {
                            // We will assume that the supe classes and interfaces have been loaded from now on.
                            // Therefore we just have to create the real loaded class.
                            let superclass = if let Some(superclass_name) = &loading.super_class {
                                match self.classes_by_name.get(superclass_name) {
                                    Some(class) => match class {
                                        LoadedClass::Loaded(class) => Some(class.clone()),
                                        LoadedClass::Loading(_) => {
                                            return Err(DerivingError::SuperClassNotLoaded.into())
                                        }
                                    },
                                    None => return Err(DerivingError::SuperClassNotLoaded.into()),
                                }
                            } else {
                                None
                            };

                            let mut interfaces = Vec::new();
                            for interface_name in &loading.interfaces {
                                match self.classes_by_name.get(interface_name) {
                                    Some(class) => match class {
                                        LoadedClass::Loaded(class) => {
                                            interfaces.push(class.clone())
                                        }
                                        LoadedClass::Loading(_) => {
                                            return Err(
                                                DerivingError::SuperInterfaceNotLoaded.into()
                                            )
                                        }
                                    },
                                    None => {
                                        return Err(DerivingError::SuperInterfaceNotLoaded.into())
                                    }
                                }
                            }

                            let class = Class {
                                id: loading.class_id,
                                name: loading.class_name.clone(),
                                superclass: superclass.map(|x| x.id).unwrap_or(ClassId(0)),
                                interfaces: interfaces.iter().map(|x| x.id).collect(),
                                flags: loading.flags,
                                constant_pool: loading.constant_pool.clone(),
                                fields: loading.fields.clone(),
                                methods: loading.methods.clone(),
                                initialized: OnceCell::new(),
                            };
                            class.initialized.set(false).unwrap();

                            let loaded_class = LoadedClass::Loaded(class);

                            // Update the class manager with the fully loaded class.
                            let _ = self
                                .classes_by_name
                                .insert(class_name.clone(), loaded_class.clone());
                            let _ = self
                                .classes_by_id
                                .insert(loading.class_id, loaded_class.clone());

                            // Invoke the class initializer.
                            log::debug!("Invoking class initializer for {}", &loading.class_name);
                            if let Err(err) =
                                self.execute_class_init(&mut init_thread, &loading.class_id)
                            {
                                return Err(ClassLoadingError::InitializerError { source: err });
                            }
                        }
                    }
                } else {
                    let classfile = self.class_loader.load_classfile(&class_name)?;
                    let loaded_class = self.resolve_class(classfile)?;
                    self.classes_by_name
                        .insert(class_name.clone(), loaded_class.clone());
                    self.classes_by_id
                        .insert(loaded_class.id(), loaded_class.clone());
                    self.next_class_id = ClassId(self.next_class_id.0 + 1);
                    if let LoadedClass::Loading(loading) = loaded_class {
                        stack.push(class_name);
                        for dependency in &loading.class_dependencies {
                            stack.push(dependency.clone());
                        }
                    }
                }
            }
        }
        Ok(self.classes_by_name.get(class_name).unwrap())
    }

    pub fn resolve_class(
        &mut self,
        classfile: ClassFile,
    ) -> Result<LoadedClass, ClassLoadingError> {
        let class_name = classfile.class_name()?;
        let class_id = self.next_class_id;
        let super_name = classfile.super_class_name()?;
        //let flags = classfile.access_flags();
        let interfaces = classfile.super_interfaces_names()?;
        let mut dependencies = Vec::new();
        if let Some(ref super_name) = super_name {
            dependencies.push(super_name.to_string());
        }
        for interface in interfaces.iter() {
            dependencies.push(interface.to_string());
        }

        if dependencies.contains(&(class_name.to_string())) {
            return Err(DerivingError::CircularDependency {
                class_name: class_name.to_string(),
            }
            .into());
        }

        // Preloading Class/Interface referenced in the ConstantPool.
        // Exception for java/lang/Object, which is the root of the class hierarchy.
        if dbg!(&class_name) != "java/lang/Object" {
            for entry in classfile.constant_pool().inner() {
                if let ConstantPoolEntry::Entry(ConstantPoolInfo::ClassInfo(class_ref)) = entry {
                    let Some(class_name) = classfile
                        .constant_pool()
                        .get_utf8_string(class_ref.name_index())
                    else {
                        log::error!(
                            "Invalid class name reference at index {}, found: {:?}",
                            class_ref.name_index(),
                            classfile.constant_pool().get(class_ref.name_index())
                        );
                        return Err(ClassLoadingError::ConstantPoolLoadingError {
                            source: ConstantPoolError::InvalidClassNameReference {
                                index: class_ref.name_index(),
                            },
                        });
                    };
                    if self.classes_by_name.contains_key(&class_name.to_string()) {
                        continue;
                    }
                    if dependencies.contains(&class_name.to_string()) {
                        continue;
                    }
                    dependencies.push(class_name.to_string());
                }
            }
        }

        Ok(LoadedClass::Loading(LoadingClass {
            class_id,
            class_name: class_name.to_string(),
            super_class: super_name.map(String::from),
            interfaces: interfaces.iter().map(|x| x.to_string()).collect(),
            flags: classfile.access_flags().clone(),
            constant_pool: ConstantPool::from_classfile(self, classfile.constant_pool())?,
            class_dependencies: dependencies,
            fields: classfile
                .fields()
                .iter()
                .map(|field| {
                    class::Field::try_from_classfile(self, classfile.constant_pool(), field)
                })
                .collect::<Result<Vec<_>, _>>()?,
            methods: classfile
                .methods()
                .iter()
                .map(|method| {
                    class::Method::try_from_classfile(self, classfile.constant_pool(), method)
                })
                .collect::<Result<Vec<_>, _>>()?,
        }))
    }
}

#[derive(Debug, Clone)]
pub enum LoadedClass {
    Loaded(Class),
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

#[derive(Debug, Clone)]
pub struct LoadingClass {
    pub class_id: ClassId,
    pub class_name: String,
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,
    pub flags: FlagSet<ClassAccessFlags>,
    pub constant_pool: ConstantPool,
    pub class_dependencies: Vec<String>,
    pub fields: Vec<class::Field>,
    pub methods: Vec<class::Method>,
}