use reader::base::classfile::ClassAccessFlags;
use crate::constant_pool::ConstantPool;

/// Runtime identifier for a class.
///
/// This is used to identify a class at runtime, and is used as a key in the
/// éclass table".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassId(pub usize);

/// Runtime representation of a class.
///
/// This is the main data structure used to represent a class at runtime.
#[derive(Debug)]
pub struct Class<'a> {
    pub id: ClassId,
    pub name: &'a str,

    pub constant_pool: ConstantPool,

    pub superclass: Option<ClassRef<'a>>,
    pub interfaces: Vec<ClassRef<'a>>,

    pub flags: ClassAccessFlags,
    // pub fields: Vec<Field<'a>>,
    // pub methods: Vec<Method<'a>>,
}

/// Runtime representation of a class reference.
pub type ClassRef<'a> = &'a Class<'a>;

