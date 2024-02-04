use dumpster::Collectable;
use reader::base::constant_pool::ConstantPoolEntry as ClassfileConstantPoolEntry;
use reader::base::constant_pool::ConstantPoolInfo as ClassfileConstantPoolInfo;
use reader::base::ConstantPool as ClassfileConstantPool;
use reader::descriptor;
use reader::descriptor::ClassName;
use reader::descriptor::FieldDescriptor;
use reader::descriptor::FieldType;
use reader::descriptor::MethodDescriptor;
use snafu::{ResultExt, Snafu};

use crate::class::ClassId;
use crate::class_manager::ClassManager;
use crate::opcode::InstructionError;

/// Runtime representation of the constant pool.
#[derive(Debug, Clone)]
pub struct ConstantPool {
    /// A mapping from the constant pool index to the index of the corresponding
    /// entry in the `entries` vector.
    ///
    /// Note that the index 0 is not used, as the constant pool index starts at
    /// 1.
    pub mappings: Vec<usize>,
    pub entries: Vec<ConstantPoolEntry>,
}

impl ConstantPool {
    fn new(entries: Vec<ConstantPoolEntry>) -> Self {
        Self {
            mappings: vec![0],
            entries,
        }
    }

    pub fn get(&self, index: usize) -> Option<&ConstantPoolEntry> {
        if index == 0 || index >= self.mappings.len() {
            return None;
        }
        let map = self.mappings.get(index)?;
        self.entries.get(*map)
    }

    pub fn get_field_ref(&self, index: usize) -> Option<&ConstantPoolEntry> {
        let entry = self.get(index)?;
        match entry {
            ConstantPoolEntry::FieldReference { .. } => Some(entry),
            _ => None,
        }
    }

    pub fn get_method_ref(&self, index: usize) -> Option<&ConstantPoolEntry> {
        let entry = self.get(index)?;
        match entry {
            ConstantPoolEntry::MethodReference { .. } => Some(entry),
            _ => None,
        }
    }

    pub fn get_class_ref(&self, index: usize) -> Option<&ConstantPoolEntry> {
        let entry = self.get(index)?;
        match entry {
            ConstantPoolEntry::ClassReference(_) => Some(entry),
            _ => None,
        }
    }

    pub fn get_array_ref(&self, index: usize) -> Option<&ConstantPoolEntry> {
        let entry = self.get(index)?;
        match entry {
            ConstantPoolEntry::ArrayReference(_) => Some(entry),
            _ => None,
        }
    }

    fn append(&mut self, entry: ConstantPoolEntry) {
        self.entries.push(entry);
        self.mappings.push(self.entries.len() - 1);
    }

    pub fn from_classfile(
        cm: &mut ClassManager,
        classfile_cp: &ClassfileConstantPool,
    ) -> Result<Self, ConstantPoolError> {
        let mut cp = ConstantPool::new(vec![]);
        for entry in classfile_cp.inner() {
            if let ClassfileConstantPoolEntry::Entry(ref entry) = entry {
                match entry {
                    ClassfileConstantPoolInfo::IntegerInfo(info) => {
                        cp.append(ConstantPoolEntry::IntegerConstant(info.value()));
                    }
                    ClassfileConstantPoolInfo::FloatInfo(info) => {
                        cp.append(ConstantPoolEntry::FloatConstant(info.value()));
                    }
                    ClassfileConstantPoolInfo::LongInfo(info) => {
                        cp.append(ConstantPoolEntry::LongConstant(info.value()));
                    }
                    ClassfileConstantPoolInfo::DoubleInfo(info) => {
                        cp.append(ConstantPoolEntry::DoubleConstant(info.value()));
                    }
                    ClassfileConstantPoolInfo::FieldRefInfo(info) => {
                        let class_name = classfile_cp
                            .get_class_name(info.class_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidClassNameReference {
                                index: info.class_index as usize,
                            })?;
                        let (field_name, field_descriptor) = classfile_cp
                            .get_name_and_type(info.name_and_type_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidFieldReference {
                                index: info.name_and_type_index as usize,
                            })?;
                        let implementor = cm
                            .id_of_class(&class_name)
                            .ok_or_else(|| {
                                log::debug!(target:"rt::constantpool::fieldref", "Class loading failure (name: {})", &class_name);
                                ConstantPoolError::ClassLoadingFailure {
                                    class_name: class_name.to_string(),
                                    context: Some(format!("FieldRefInfo (name: {}, descriptor: {}) at index {}", field_name, field_descriptor, info.name_and_type_index as usize))
                                }
                            })?;
                        let descriptor =
                            descriptor::parse_field_descriptor(&field_descriptor.to_owned())
                                .map_err(|err| ConstantPoolError::InvalidDescriptor {
                                    index: info.name_and_type_index as usize,
                                    source: err,
                                })?;

                        cp.append(ConstantPoolEntry::FieldReference {
                            field_name: field_name.to_string(),
                            field_descriptor: descriptor,
                            implementor,
                        });
                    }
                    ClassfileConstantPoolInfo::MethodRefInfo(info) => {
                        let class_name = classfile_cp
                            .get_class_name(info.class_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidClassNameReference {
                                index: info.class_index as usize,
                            })?;
                        let (method_name, method_descriptor) = classfile_cp
                            .get_name_and_type(info.name_and_type_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidFieldReference {
                                index: info.name_and_type_index as usize,
                            })?;
                        let implementor = cm
                            .id_of_class(&class_name)
                            .ok_or_else(|| {
                                log::debug!(target:"rt::constantpool::methodref", "Class loading failure (name: {})", &class_name);
                                ConstantPoolError::ClassLoadingFailure {
                                    class_name: class_name.to_string(),
                                    context: Some(format!("MethodRefInfo (name: {}, descriptor: {}) at index {}", method_name, method_descriptor, info.name_and_type_index as usize))
                                }
                            })?;
                        let descriptor =
                            descriptor::parse_method_descriptor(&&method_descriptor.to_owned())
                                .map_err(|err| ConstantPoolError::InvalidDescriptor {
                                    index: info.name_and_type_index as usize,
                                    source: err,
                                })?;

                        cp.append(ConstantPoolEntry::MethodReference {
                            method_name: method_name.to_string(),
                            method_descriptor: descriptor,
                            implementor,
                        });
                    }
                    ClassfileConstantPoolInfo::InterfaceMethodRefInfo(info) => {
                        let class_name = classfile_cp
                            .get_class_name(info.class_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidClassNameReference {
                                index: info.class_index as usize,
                            })?;
                        let (method_name, method_descriptor) = classfile_cp
                            .get_name_and_type(info.name_and_type_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidFieldReference {
                                index: info.name_and_type_index as usize,
                            })?;
                        let implementor = cm
                            .id_of_class(&class_name)
                            .ok_or_else(|| {
                                log::debug!(target:"rt::constantpool::interfacemethodref", "Class loading failure (name: {})", &class_name);
                                ConstantPoolError::ClassLoadingFailure {
                                    class_name: class_name.to_string(),
                                    context: Some(format!("InterfaceMethodRefInfo (name: {}, descriptor: {}) at index {}", method_name, method_descriptor, info.name_and_type_index as usize))
                                }
                            })?;
                        let descriptor =
                            descriptor::parse_method_descriptor(&&method_descriptor.to_owned())
                                .map_err(|err| ConstantPoolError::InvalidDescriptor {
                                    index: info.name_and_type_index as usize,
                                    source: err,
                                })?;

                        cp.append(ConstantPoolEntry::InterfaceMethodReference {
                            method_name: method_name.to_string(),
                            method_descriptor: descriptor,
                            implementor,
                        });
                    }
                    ClassfileConstantPoolInfo::ClassInfo(info) => {
                        let class_name = classfile_cp
                            .get_utf8_string(info.name_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidClassNameReference {
                                index: info.name_index as usize,
                            })?;
                        if class_name.starts_with("[") {
                            let field_type = descriptor::parse_field_descriptor(&class_name)
                                .map_err(|err| ConstantPoolError::InvalidDescriptor {
                                    index: info.name_index as usize,
                                    source: err,
                                })?;
                            cp.append(ConstantPoolEntry::ArrayReference(
                                field_type.field_type().clone(),
                            ));
                        } else {
                            let class_id = cm
                                .id_of_class(&class_name)
                                .ok_or_else(|| {
                                    log::debug!(target:"rt::constantpool::classinfo", "Class loading failure (name: {})", &class_name);
                                    ConstantPoolError::ClassLoadingFailure {
                                        class_name: class_name.to_string(),
                                        context: Some(format!("ClassInfo at index {}", info.name_index as usize))
                                    }
                                })?;
                            cp.append(ConstantPoolEntry::ClassReference(class_id));
                        }
                    }
                    _ => {
                        log::trace!("Constant pool entry not necessary or unimplemented, ignored in RtConstantPool: {:?}", entry);
                        cp.mappings.push(0);
                    }
                }
            } else {
                // Tombstone, this entry is not used.
                cp.mappings.push(0);
            }
        }
        Ok(cp)
    }
}

#[derive(Debug, Snafu)]
pub enum ConstantPoolError {
    #[snafu(display("Invalid UTF-8 string reference, entry index: {}", index))]
    InvalidUtf8StringReference { index: usize },

    #[snafu(display("Invalid field reference, entry index: {} (either the entry at this index is not a FieldRef or the component of the entry are invalid)", index))]
    InvalidFieldReference { index: usize },

    #[snafu(display("Invalid constant reference, entry index: {}", index))]
    InvalidConstantReference { index: usize },

    #[snafu(display("Invalid descriptor, entry index: {}, source: {}", index, source))]
    InvalidDescriptor {
        index: usize,
        source: descriptor::DescriptorError,
    },

    #[snafu(display("Invalid classname reference, entry index: {}", index))]
    InvalidClassNameReference { index: usize },

    #[snafu(display("Loading failure of a class/interface reference, name: {}, context: {}", class_name, context.as_ref().unwrap_or(&"<unknown>".to_string())))]
    ClassLoadingFailure {
        class_name: String,
        context: Option<String>,
    },
}

/// Runtime representation of a constant pool entry.
#[derive(Debug, Clone)]
pub enum ConstantPoolEntry {
    IntegerConstant(i32),
    FloatConstant(f32),
    LongConstant(i64),
    DoubleConstant(f64),
    // TODO: String constant should be a reference to a java String object.
    // StringConstant(String),
    // TODO: Implement the rest of the constant pool entries, in particular
    // the symbolic references (class, field, method, interface method, ...).
    FieldReference {
        field_name: String,
        field_descriptor: FieldDescriptor,
        implementor: ClassId,
    },
    MethodReference {
        method_name: String,
        method_descriptor: MethodDescriptor,
        implementor: ClassId,
    },
    InterfaceMethodReference {
        method_name: String,
        method_descriptor: MethodDescriptor,
        implementor: ClassId,
    },
    ClassReference(ClassId),
    ArrayReference(FieldType),
    // MethodHandleReference(MethodHandleReference),
    // MethodTypeReference(String),
}
