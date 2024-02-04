use crate::slot::Slot;
use std::{cell::OnceCell, io::Cursor};

use crate::{
    class_loader::ClassLoadingError,
    class_manager::ClassManager,
    constant_pool::{ConstantPool, ConstantPoolError},
};
use dumpster::Collectable;
use flagset::FlagSet;
use reader::{
    base::classfile::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags},
    BinRead,
};
use reader::{
    base::{
        attribute_info::{CodeAttribute, ConstantValueAttribute},
        classfile,
        constant_pool::ConstantPoolInfo as ClassfileConstantPoolInfo,
        AttributeInfo, ConstantPool as ClassfileConstantPool,
    },
    descriptor::{self, FieldDescriptor, MethodDescriptor},
};

/// Runtime identifier for a class.
///
/// This is used to identify a class at runtime, and is used as a key in the
/// éclass table".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Collectable)]
pub struct ClassId(pub usize);

/// Runtime representation of a class.
///
/// This is the main data structure used to represent a class at runtime.
#[derive(Debug, Clone)]
pub struct Class {
    pub id: ClassId,
    pub name: String,

    pub constant_pool: ConstantPool,
    pub superclass: Option<ClassId>,
    pub interfaces: Vec<ClassId>,
    pub flags: FlagSet<ClassAccessFlags>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    /// Whether the class has been initialized.
    ///
    /// Basically ensure the `<clinit>` method has been executed, or not.
    /// This is particularly useful for ensuring final static fields are set only once.
    pub initialized: OnceCell<bool>,
}

impl Class {
    pub fn get_method(
        &self,
        name: &str,
        descriptor: &MethodDescriptor,
    ) -> Option<(usize, &Method)> {
        self.methods
            .iter()
            .enumerate()
            .find(|method| method.1.name == name && method.1.descriptor == *descriptor)
    }

    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|field| field.name == name)
    }

    pub fn get_mut_field(&mut self, name: &str) -> Option<&mut Field> {
        self.fields.iter_mut().find(|field| field.name == name)
    }

    pub fn get_field_by_index(&self, index: usize) -> Option<&Field> {
        self.fields.get(index)
    }

    pub fn get_method_by_index(&self, index: usize) -> Option<&Method> {
        self.methods.get(index)
    }

    pub fn index_of_method(&self, name: &str, descriptor: &MethodDescriptor) -> Option<usize> {
        self.methods
            .iter()
            .position(|method| method.name == name && method.descriptor == *descriptor)
    }

    pub fn index_of_field(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|field| field.name == name)
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub descriptor: FieldDescriptor,
    pub flags: FlagSet<FieldAccessFlags>,
    pub value: Slot,
    pub attributes: Vec<FieldAttribute>,
}

impl Field {
    pub fn try_from_classfile(
        cm: &mut ClassManager,
        cp: &ClassfileConstantPool,
        fi: &classfile::FieldInfo,
    ) -> Result<Self, ClassLoadingError> {
        let name = cp.get_utf8_string(fi.name_index as usize).ok_or_else(|| {
            ConstantPoolError::InvalidUtf8StringReference {
                index: fi.name_index as usize,
            }
        })?;
        let descriptor = cp
            .get_utf8_string(fi.descriptor_index as usize)
            .ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference {
                index: fi.descriptor_index as usize,
            })?;

        let descriptor = descriptor::parse_field_descriptor(&descriptor.to_string())?;

        let attributes: Vec<FieldAttribute> = fi
            .attributes
            .iter()
            .map(|attr| parse_field_attribute(cm, cp, attr))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        let mut value = Slot::Tombstone;
        if let Some(cv) = attributes
            .iter()
            .find(|x| matches!(x, &&FieldAttribute::ConstantValue { .. }))
        {
            if let FieldAttribute::ConstantValue { value: cv } = cv {
                value = cv.clone().into();
            }
        }

        let flags = fi.access_flags.clone();

        Ok(Self {
            name: name.to_string(),
            value,
            descriptor: descriptor,
            attributes,
            flags,
        })
    }

    /// Get the value of the field.
    pub fn get_value(&self) -> Option<&Slot> {
        // TODO: Check if the field is static
        Some(&self.value)
    }

    /// Get flags of the field.
    pub fn get_flags(&self) -> &FlagSet<FieldAccessFlags> {
        &self.flags
    }

    /// Check if the field is static.
    pub fn is_static(&self) -> bool {
        self.flags.contains(FieldAccessFlags::Static)
    }

    /// Check if the field is final.
    pub fn is_final(&self) -> bool {
        self.flags.contains(FieldAccessFlags::Final)
    }
}

#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub descriptor: MethodDescriptor,
    pub flags: FlagSet<MethodAccessFlags>,
    pub attributes: Vec<MethodAttribute>,
}

impl Method {
    pub fn try_from_classfile(
        cm: &mut ClassManager,
        cp: &ClassfileConstantPool,
        mi: &classfile::MethodInfo,
    ) -> Result<Self, ClassLoadingError> {
        let name = cp.get_utf8_string(mi.name_index as usize).ok_or_else(|| {
            ConstantPoolError::InvalidUtf8StringReference {
                index: mi.name_index as usize,
            }
        })?;
        let descriptor = cp
            .get_utf8_string(mi.descriptor_index as usize)
            .ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference {
                index: mi.descriptor_index as usize,
            })?;

        let descriptor = descriptor::parse_method_descriptor(&descriptor.to_string())?;

        let attributes: Vec<MethodAttribute> = mi
            .attributes
            .iter()
            .map(|attr| parse_method_attribute(cm, cp, attr))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        let flags = mi.access_flags.clone();

        Ok(Self {
            name: name.to_string(),
            descriptor: descriptor,
            attributes,
            flags,
        })
    }

    pub fn get_code(&self) -> Option<&MethodCode> {
        self.attributes.iter().find_map(|attr| match attr {
            MethodAttribute::Code(code) => Some(code),
            _ => None,
        })
    }

    pub fn get_flags(&self) -> &FlagSet<MethodAccessFlags> {
        &self.flags
    }

    pub fn is_static(&self) -> bool {
        self.flags.contains(MethodAccessFlags::Static)
    }

    pub fn is_native(&self) -> bool {
        self.flags.contains(MethodAccessFlags::Native)
    }

    pub fn is_abstract(&self) -> bool {
        self.flags.contains(MethodAccessFlags::Abstract)
    }

    pub fn is_final(&self) -> bool {
        self.flags.contains(MethodAccessFlags::Final)
    }

    pub fn is_synchronized(&self) -> bool {
        self.flags.contains(MethodAccessFlags::Synchronized)
    }

    pub fn is_private(&self) -> bool {
        self.flags.contains(MethodAccessFlags::Private)
    }

    pub fn is_public(&self) -> bool {
        self.flags.contains(MethodAccessFlags::Public)
    }

    pub fn is_protected(&self) -> bool {
        self.flags.contains(MethodAccessFlags::Protected)
    }
}

#[derive(Debug, Collectable, Clone)]
pub enum FieldAttribute {
    ConstantValue { value: ConstantValue },
    Synthetic,
    Deprecated,
}

#[derive(Debug, Collectable, Clone)]
pub enum MethodAttribute {
    Code(MethodCode),
    Synthetic,
    Deprecated,
}

#[derive(Debug, Collectable, Clone)]
pub struct MethodCode {
    pub max_stack: u16,
    pub max_locals: u16,
    pub instructions: Vec<u8>,
    // TODO: exception_table: Vec<ExceptionTableEntry>,
    // TODO: attributes: Vec<CodeAttribute>,
}

#[derive(Debug, Collectable, Clone)]
pub enum ConstantValue {
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

pub fn parse_field_attribute(
    _cm: &mut ClassManager,
    cp: &ClassfileConstantPool,
    attribute: &AttributeInfo,
) -> Result<Option<FieldAttribute>, ClassLoadingError> {
    let name = cp
        .get_utf8_string(attribute.attribute_name_index as usize)
        .ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference {
            index: attribute.attribute_name_index as usize,
        })?;
    match name.as_ref() {
        "ConstantValue" => {
            let mut reader = Cursor::new(attribute.info.as_slice());
            let cvattr = ConstantValueAttribute::read(&mut reader)?;
            let value = cp
                .get_info(cvattr.constant_value_index as usize)
                .ok_or_else(|| ConstantPoolError::InvalidConstantReference {
                    index: cvattr.constant_value_index as usize,
                })?;
            match value {
                ClassfileConstantPoolInfo::IntegerInfo(info) => {
                    Ok(Some(FieldAttribute::ConstantValue {
                        value: ConstantValue::Integer(info.value()),
                    }))
                }
                ClassfileConstantPoolInfo::LongInfo(info) => {
                    Ok(Some(FieldAttribute::ConstantValue {
                        value: ConstantValue::Long(info.value()),
                    }))
                }
                ClassfileConstantPoolInfo::FloatInfo(info) => {
                    Ok(Some(FieldAttribute::ConstantValue {
                        value: ConstantValue::Float(info.value()),
                    }))
                }
                ClassfileConstantPoolInfo::DoubleInfo(info) => {
                    Ok(Some(FieldAttribute::ConstantValue {
                        value: ConstantValue::Double(info.value()),
                    }))
                }
                _ => unimplemented!("ConstantValue attribute with type: {:?}", value),
            }
        }
        "Synthetic" => Ok(Some(FieldAttribute::Synthetic)),
        "Deprecated" => Ok(Some(FieldAttribute::Deprecated)),
        _ => {
            log::debug!(
                "Field attribute not implemented/unknown, ignored: {:?}",
                &name
            );
            Ok(None)
        }
    }
}

pub fn parse_method_attribute(
    _cm: &mut ClassManager,
    cp: &ClassfileConstantPool,
    attribute: &AttributeInfo,
) -> Result<Option<MethodAttribute>, ClassLoadingError> {
    let name = cp
        .get_utf8_string(attribute.attribute_name_index as usize)
        .ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference {
            index: attribute.attribute_name_index as usize,
        })?;
    match name.as_ref() {
        "Code" => {
            let mut reader = Cursor::new(attribute.info.as_slice());
            let codeattr = CodeAttribute::read(&mut reader)?;
            // TODO: let attributes = codeattr.attributes.iter().map(|attr| parse_code_attribute(cm, cp, attr)).collect::<Result<Vec<_>, _>>()?.into_iter().flatten().collect();
            Ok(Some(MethodAttribute::Code(MethodCode {
                max_stack: codeattr.max_stack,
                max_locals: codeattr.max_locals,
                instructions: codeattr.code,
            })))
        }
        "Synthetic" => Ok(Some(MethodAttribute::Synthetic)),
        "Deprecated" => Ok(Some(MethodAttribute::Deprecated)),
        _ => {
            log::debug!(
                "Method attribute not implemented/unknown, ignored: {:?}",
                &name
            );
            Ok(None)
        }
    }
}
