use std::io::Cursor;

use crate::{constant_pool::{ConstantPool, ConstantPoolError}, class_manager::ClassManager, class_loader::ClassLoadingError};
use dumpster::{sync::Gc, Collectable};
use reader::base::{classfile, ConstantPool as ClassfileConstantPool, AttributeInfo, attribute_info::{ConstantValueAttribute, CodeAttribute}, constant_pool::ConstantPoolInfo as ClassfileConstantPoolInfo};
use reader::BinRead;

/// Runtime identifier for a class.
///
/// This is used to identify a class at runtime, and is used as a key in the
/// éclass table".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Collectable)]
pub struct ClassId(pub usize);

/// Runtime representation of a class.
///
/// This is the main data structure used to represent a class at runtime.
#[derive(Debug, Collectable, Clone)]
pub struct Class {
    pub id: ClassId,
    pub name: String,

    pub constant_pool: ConstantPool,
    pub superclass: Option<ClassRef>,
    pub interfaces: Vec<ClassRef>,
    // pub flags: ClassAccessFlags,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
}

/// Runtime representation of a class reference.
pub type ClassRef = Gc<Class>;

#[derive(Debug, Collectable, Clone)]
pub struct Field {
    pub name: String,
    pub descriptor: String,
    // pub flags: FieldAccessFlags,
    pub attributes: Vec<FieldAttribute>,
}

impl Field {
    pub fn try_from_classfile(cm: &mut ClassManager, cp: &ClassfileConstantPool, fi: &classfile::FieldInfo) -> Result<Self, ClassLoadingError> {
        let name = cp.get_utf8_string(fi.name_index as usize).ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference { index: fi.name_index as usize })?;
        let descriptor = cp.get_utf8_string(fi.descriptor_index as usize).ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference { index: fi.descriptor_index as usize })?;
        let attributes : Vec<FieldAttribute>  = fi.attributes.iter().map(|attr| parse_field_attribute(cm, cp, attr)).collect::<Result<Vec<_>, _>>()?.into_iter().flatten().collect();
        Ok(Self {
            name: name.to_string(),
            descriptor: descriptor.to_string(),
            attributes,
        })
    }
}

#[derive(Debug, Collectable, Clone)]
pub struct Method {
    pub name: String,
    pub descriptor: String,
    // pub flags: MethodAccessFlags,
    pub attributes: Vec<MethodAttribute>,
}

impl Method {
    pub fn try_from_classfile(cm: &mut ClassManager, cp: &ClassfileConstantPool, mi: &classfile::MethodInfo) -> Result<Self, ClassLoadingError> {
        let name = cp.get_utf8_string(mi.name_index as usize).ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference { index: mi.name_index as usize })?;
        let descriptor = cp.get_utf8_string(mi.descriptor_index as usize).ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference { index: mi.descriptor_index as usize })?;
        let attributes : Vec<MethodAttribute>  = mi.attributes.iter().map(|attr| parse_method_attribute(cm, cp, attr)).collect::<Result<Vec<_>, _>>()?.into_iter().flatten().collect();
        Ok(Self {
            name: name.to_string(),
            descriptor: descriptor.to_string(),
            attributes,
        })
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
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
        // TODO: exception_table: Vec<ExceptionTableEntry>,
        // TODO: attributes: Vec<CodeAttribute>,
    },
    Synthetic,
    Deprecated,
}

#[derive(Debug, Collectable, Clone)]
pub enum ConstantValue {
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

pub fn parse_field_attribute(cm: &mut ClassManager, cp: &ClassfileConstantPool, attribute: &AttributeInfo) -> Result<Option<FieldAttribute>, ClassLoadingError> {
    let name = cp.get_utf8_string(attribute.attribute_name_index as usize).ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference { index: attribute.attribute_name_index as usize })?;
    match name.as_ref() {
        "ConstantValue" => {
            let mut reader = Cursor::new(attribute.info.as_slice());
            let cvattr = ConstantValueAttribute::read(&mut reader)?;
            let value = cp.get_info(cvattr.constant_value_index as usize).ok_or_else(|| ConstantPoolError::InvalidConstantReference { index: cvattr.constant_value_index as usize })?;
            match value {
                ClassfileConstantPoolInfo::IntegerInfo(info) => Ok(Some(FieldAttribute::ConstantValue { value: ConstantValue::Integer(info.value()) })),
                _ => unimplemented!(),
            }
        },
        "Synthetic" => Ok(Some(FieldAttribute::Synthetic)),
        "Deprecated" => Ok(Some(FieldAttribute::Deprecated)),
        _ => {
            log::debug!("Field attribute not implemented/unknown, ingnored: {:?}", &name);
            Ok(None)
        }
    }
}

pub fn parse_method_attribute(cm: &mut ClassManager, cp: &ClassfileConstantPool, attribute: &AttributeInfo) -> Result<Option<MethodAttribute>, ClassLoadingError> {
    let name = cp.get_utf8_string(attribute.attribute_name_index as usize).ok_or_else(|| ConstantPoolError::InvalidUtf8StringReference { index: attribute.attribute_name_index as usize })?;
    match name.as_ref() {
        "Code" => {
            let mut reader = Cursor::new(attribute.info.as_slice());
            let codeattr = CodeAttribute::read(&mut reader)?;
            // TODO: let attributes = codeattr.attributes.iter().map(|attr| parse_code_attribute(cm, cp, attr)).collect::<Result<Vec<_>, _>>()?.into_iter().flatten().collect();
            Ok(Some(MethodAttribute::Code {
                max_stack: codeattr.max_stack,
                max_locals: codeattr.max_locals,
                code: codeattr.code,
            }))
        },
        "Synthetic" => Ok(Some(MethodAttribute::Synthetic)),
        "Deprecated" => Ok(Some(MethodAttribute::Deprecated)),
        _ => {
            log::debug!("Method attribute not implemented/unknown, ingnored: {:?}", &name);
            Ok(None)
        }
    }
}