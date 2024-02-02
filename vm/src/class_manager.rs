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

    /// The mapping between class names and their ID.
    pub name_map: HashMap<String, ClassId>,

    /// The next class ID to use.
    next_class_id: ClassId,
}

impl ClassManager {
    pub fn new(class_loader: ClassLoader) -> Self {
        Self {
            class_loader,
            classes_by_id: HashMap::new(),
            name_map: HashMap::new(),
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

    pub fn get_class_by_name(&self, name: &str) -> Option<&LoadedClass> {
        self.name_map.get(name).and_then(|id| self.classes_by_id.get(id))
    }

    pub fn id_of_class(&self, name: &str) -> Option<ClassId> {
        self.name_map.get(name).cloned()
    }

    pub fn acquire_class_id(&mut self) -> ClassId {
        let id = self.next_class_id;
        self.next_class_id = ClassId(self.next_class_id.0 + 1);
        id
    }

    pub fn get_or_resolve_class(
        &mut self,
        class_name: &str,
    ) -> Result<&LoadedClass, ClassLoadingError> {
        if !self.name_map.contains_key(class_name) {
            let mut init_thread = Thread::new();
            let mut stack: Vec<String> = Vec::new();
            stack.push(class_name.to_string());
            while let Some(class_name) = stack.pop() {
                log::debug!("Resolving class: {}", &class_name);
                if let Some(class) = self.get_class_by_name(&class_name) {
                    let class = class.clone();
                    match class {
                        LoadedClass::Loaded(_) => (),
                        LoadedClass::Resolved(resolved) => {
                            // Run the loading of the dependencies.
                            let mut unresolved = Vec::new();
                            for dependency in &resolved.class_dependencies {
                                if !self.name_map.contains_key(dependency) {
                                    unresolved.push(dependency.clone());
                                }
                            }
                            stack.push(class_name.clone());
                            for dependency in unresolved {
                                let classfile = self.class_loader.load_classfile(&dependency)?;
                                self.resolve_class(classfile)?;
                                stack.push(dependency);
                            }

                            // Once the dependencies are resolved (all of them has at least a ClassId),
                            // we can create the LoadingClass, and construct the constantpool, fields and methods.
                            let loaded_class = LoadedClass::Loading(LoadingClass {
                                class_id: resolved.class_id,
                                class_name: class_name.to_string(),
                                super_class: resolved.super_class,
                                interfaces: resolved.interfaces,
                                flags: resolved.classfile.access_flags().clone(),
                                constant_pool: ConstantPool::from_classfile(self, resolved.classfile.constant_pool())?,
                                fields: resolved.classfile
                                    .fields()
                                    .iter()
                                    .map(|field| {
                                        class::Field::try_from_classfile(self, resolved.classfile.constant_pool(), field)
                                    })
                                    .collect::<Result<Vec<_>, _>>()?,
                                methods: resolved.classfile
                                    .methods()
                                    .iter()
                                    .map(|method| {
                                        class::Method::try_from_classfile(self, resolved.classfile.constant_pool(), method)
                                    })
                                    .collect::<Result<Vec<_>, _>>()?,
                            });

                            // Update the class manager with the loading class.
                            self.classes_by_id.insert(loaded_class.id(), loaded_class);
                        }
                        LoadedClass::Loading(loading) => {
                            // We will assume that the supe classes and interfaces have been loaded from now on.
                            // Therefore we just have to create the real loaded class.
                            let superclass = if let Some(superclass_name) = &loading.super_class {
                                match self.get_class_by_name(superclass_name) {
                                    Some(class) => match class {
                                        LoadedClass::Loaded(class) => Some(class.clone()),
                                        LoadedClass::Loading(_) | LoadedClass::Resolved(_) => {
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
                                match self.get_class_by_name(interface_name) {
                                    Some(class) => match class {
                                        LoadedClass::Loaded(class) => {
                                            interfaces.push(class.clone())
                                        }
                                        LoadedClass::Loading(_) | LoadedClass::Resolved(_) => {
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
                            let _ = self.name_map
                                .insert(class_name.clone(), loaded_class.id());
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
                    self.resolve_class(classfile)?;
                    stack.push(class_name);
                }
            }
        }
        Ok(self.get_class_by_name(class_name).unwrap())
    }


    /// Load a class from a classfile, and resolve its dependencies.
    ///
    /// This method will produces a ResolvedClass, with all its dependencies calculated.
    pub fn resolve_class(
        &mut self,
        classfile: ClassFile,
    ) -> Result<ClassId, ClassLoadingError> {
        let class_name = classfile.class_name()?.to_string();
        let class_id = self.acquire_class_id();
        let super_name = classfile.super_class_name()?.map(|x| x.to_string());
        //let flags = classfile.access_flags();
        let interfaces : Vec<String> = classfile.super_interfaces_names()?.iter().map(|x| x.to_string()).collect();
        let mut dependencies = Vec::new();
        if let Some(ref super_name) = super_name {
            dependencies.push(super_name.clone());
        }
        for interface in interfaces.iter() {
            dependencies.push(interface.clone());
        }

        if dependencies.contains(&(class_name.to_string())) {
            return Err(DerivingError::CircularDependency {
                class_name: class_name.to_string(),
            }
            .into());
        }

        // Construct the dependencies list of Field, Method, etc refs.
        for entry in classfile.constant_pool().inner() {
            if let ConstantPoolEntry::Entry(ConstantPoolInfo::ClassInfo(class_ref)) = entry {
                let Some(mut dep_class_name) = classfile
                    .constant_pool()
                    .get_utf8_string(class_ref.name_index())
                    .map(|x| x.to_string())
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
                if dep_class_name.len() == 0{
                    continue;
                }
                if dep_class_name.starts_with("[") {
                    // This is an array type, FUCK
                    if let Ok(descriptor) = descriptor::parse_field_descriptor(&dep_class_name) {
                        if let Some(rcn) = descriptor.get_referenced_class() {
                            dep_class_name = rcn.as_binary_name();
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                if class_name == dep_class_name {
                    continue;
                }
                if self.name_map.contains_key(&dep_class_name) {
                    continue;
                }
                if dependencies.contains(&dep_class_name) {
                    continue;
                }
                dependencies.push(dep_class_name);
            }
        }

        log::debug!("Class `{}` depends on {} other classes.", &class_name, dependencies.len());

        let class = LoadedClass::Resolved(ResovedClass {
            class_id,
            class_name: class_name.clone(),
            super_class: super_name.map(|x| x.to_string()),
            interfaces: interfaces,
            classfile,
            class_dependencies: dependencies,
        });

        self.classes_by_id.insert(class_id, class.clone());
        self.name_map.insert(class_name, class_id);

        Ok(class_id)
    }
}

#[derive(Debug, Clone)]
pub enum LoadedClass {
    Loaded(Class),
    Loading(LoadingClass),
    Resolved(ResovedClass),
}

impl LoadedClass {
    pub fn name(&self) -> &str {
        match self {
            LoadedClass::Loaded(class) => &class.name,
            LoadedClass::Loading(class) => &class.class_name,
            LoadedClass::Resolved(class) => &class.class_name,
        }
    }

    pub fn id(&self) -> ClassId {
        match self {
            LoadedClass::Loaded(class) => class.id,
            LoadedClass::Loading(class) => class.class_id,
            LoadedClass::Resolved(class) => class.class_id,
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
    pub fields: Vec<class::Field>,
    pub methods: Vec<class::Method>,
}

#[derive(Debug, Clone)]
pub struct ResovedClass {
    pub class_id: ClassId,
    pub class_name: String,
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,
    pub classfile: ClassFile,
    pub class_dependencies: Vec<String>,
}