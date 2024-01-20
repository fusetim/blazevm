use crate::constant_pool::ConstantPool;
use dumpster::{sync::Gc, Collectable};
use reader::base::classfile::ClassAccessFlags;

/// Runtime identifier for a class.
///
/// This is used to identify a class at runtime, and is used as a key in the
/// éclass table".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Collectable)]
pub struct ClassId(pub usize);

/// Runtime representation of a class.
///
/// This is the main data structure used to represent a class at runtime.
#[derive(Debug, Collectable)]
pub struct Class {
    pub id: ClassId,
    pub name: String,

    pub constant_pool: ConstantPool,
    pub superclass: Option<ClassRef>,
    pub interfaces: Vec<ClassRef>,
    // pub flags: ClassAccessFlags,
    // pub fields: Vec<Field<'a>>,
    // pub methods: Vec<Method<'a>>,
}

/// Runtime representation of a class reference.
pub type ClassRef = Gc<Class>;
