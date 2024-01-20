use dumpster::Collectable;
use reader::base::constant_pool::ConstantPoolEntry as ClassfileConstantPoolEntry;
use reader::base::constant_pool::ConstantPoolInfo as ClassfileConstantPoolInfo;
use reader::base::ConstantPool as ClassfileConstantPool;
use snafu::Snafu;

use crate::class::ClassId;
use crate::class_manager::ClassManager;

/// Runtime representation of the constant pool.
#[derive(Debug, Collectable, Clone)]
pub struct ConstantPool {
    pub entries: Vec<ConstantPoolEntry>,
}

impl ConstantPool {
    pub fn new(entries: Vec<ConstantPoolEntry>) -> Self {
        Self { entries }
    }

    pub fn get(&self, index: usize) -> Option<&ConstantPoolEntry> {
        self.entries.get(index)
    }

    pub fn append(&mut self, entry: ConstantPoolEntry) {
        self.entries.push(entry)
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
                    ClassfileConstantPoolInfo::FieldRefInfo(info) => {
                        let class_name = classfile_cp
                            .get_utf8_string(info.class_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference {
                                index: info.class_index as usize,
                            })?;
                        let (field_name, field_descriptor) = classfile_cp
                            .get_name_and_type(info.name_and_type_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidFieldReference {
                                index: info.name_and_type_index as usize,
                            })?;
                        let implementor = cm
                            .get_or_resolve_class(&class_name)
                            .map_err(|_| ConstantPoolError::ClassLoadingFailure)?;
                        cp.append(ConstantPoolEntry::FieldReference {
                            field_name: field_name.to_string(),
                            field_descriptor: field_descriptor.to_string(),
                            implementor: implementor.id(),
                        });
                    }
                    ClassfileConstantPoolInfo::MethodRefInfo(info) => {
                        let class_name = classfile_cp
                            .get_utf8_string(info.class_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference {
                                index: info.class_index as usize,
                            })?;
                        let (method_name, method_descriptor) = classfile_cp
                            .get_name_and_type(info.name_and_type_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidFieldReference {
                                index: info.name_and_type_index as usize,
                            })?;
                        let implementor = cm
                            .get_or_resolve_class(&class_name)
                            .map_err(|_| ConstantPoolError::ClassLoadingFailure)?;
                        cp.append(ConstantPoolEntry::MethodReference {
                            method_name: method_name.to_string(),
                            method_descriptor: method_descriptor.to_string(),
                            implementor: implementor.id(),
                        });
                    }
                    ClassfileConstantPoolInfo::InterfaceMethodRefInfo(info) => {
                        let class_name = classfile_cp
                            .get_utf8_string(info.class_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference {
                                index: info.class_index as usize,
                            })?;
                        let (method_name, method_descriptor) = classfile_cp
                            .get_name_and_type(info.name_and_type_index as usize)
                            .ok_or_else(|| ConstantPoolError::InvalidFieldReference {
                                index: info.name_and_type_index as usize,
                            })?;
                        let implementor = cm
                            .get_or_resolve_class(&class_name)
                            .map_err(|_| ConstantPoolError::ClassLoadingFailure)?;
                        cp.append(ConstantPoolEntry::InterfaceMethodReference {
                            method_name: method_name.to_string(),
                            method_descriptor: method_descriptor.to_string(),
                            implementor: implementor.id(),
                        });
                    }
                    _ => {
                        log::debug!("Constant pool entry not implemented, ingnored: {:?}", entry);
                    }
                }
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

    #[snafu(display("Loading failure of a class/interface reference."))]
    ClassLoadingFailure,
}

/// Runtime representation of a constant pool entry.
#[derive(Debug, Collectable, Clone)]
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
        field_descriptor: String,
        implementor: ClassId,
    },
    MethodReference {
        method_name: String,
        method_descriptor: String,
        implementor: ClassId,
    },
    InterfaceMethodReference {
        method_name: String,
        method_descriptor: String,
        implementor: ClassId,
    },
    // MethodHandleReference(MethodHandleReference),
    // MethodTypeReference(String),
}
