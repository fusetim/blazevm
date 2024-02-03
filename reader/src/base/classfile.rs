use std::borrow::Cow;

use super::{AttributeInfo, ConstantPool, DecodingError, U2, U4};
use binrw::BinRead;
use dumpster::Collectable;
use flagset::{flags, FlagSet};

/// Model of a Class Info
///
/// The classfile structure represents the entire class file read.
/// Note: One class or module is always represented by one class file.
#[derive(BinRead, Debug, Clone)]
#[br(big)]
pub struct ClassFile {
    /// Magic number identifying the class file format
    /// Value should be 0xCAFEBABE for a valid class file for
    /// Java SE 21 and under.
    magic: U4,
    /// Minor version of the class file format
    /// Should be 0 for Java 5 and above.
    minor_version: U2,
    /// Major version of the class file format
    /// Should be 1-incremented per major release of Java
    /// starting at Major 49 for Java 5.
    major_version: U2,
    // Constant pool count
    // The number of entries in the constant pool table plus one.
    // This is because the constant pool is indexed from 1 to n-1.
    constant_pool_count: U2,
    /// Constant pool, see [crate::base::constant_pool::ConstantPool].
    #[br(args(constant_pool_count - 1))]
    constant_pool: ConstantPool,
    /// Access flags
    /// Flags indicating access permissions to and properties of this class,
    /// interface or module.
    #[br(map= |x: U2| FlagSet::<ClassAccessFlags>::new_truncated(x))]
    access_flags: FlagSet<ClassAccessFlags>,
    /// Pointer to the [crate::base::constant_pool::ClassInfo] of the current class/interface in the constant pool.
    this_class: U2,
    /// Pointer to the [crate::base::constant_pool::ClassInfo] of the super class/interface in the constant pool.
    ///
    /// For a class, this is the super class of the current class. 0 if the class is
    /// [java.lang.Object].
    /// For an interface, points to the [crate::base::constant_pool::ClassInfo] of [java.lang.Object].
    super_class: U2,
    // Interfaces count
    // The number of direct super interfaces of this class or interface type.
    interfaces_count: U2,
    /// The direct super interfaces of this class or interface type.
    /// Each entry must be a valid index into the constant pool table.
    /// The order of the interfaces is significant, and should be preserved.
    #[br(count=interfaces_count)]
    interfaces: Vec<U2>,
    // Fields count
    // The number of fields of this class or interface type.
    fields_count: U2,
    /// The fields' index into the constant pool table.
    /// It only contains the fields defined by this class or interface, and not
    /// those inherited from super classes or interfaces.
    #[br(count=fields_count)]
    fields: Vec<FieldInfo>,
    // Methods count
    // The number of methods of this class or interface type.
    methods_count: U2,
    /// The method table
    #[br(count=methods_count)]
    methods: Vec<MethodInfo>,
    // Attributes count
    attributes_count: U2,
    /// Attribute table
    #[br(count=attributes_count)]
    attributes: Vec<AttributeInfo>,
}

impl ClassFile {
    /// Read a class file from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, binrw::Error> {
        let mut reader = std::io::Cursor::new(bytes);
        Self::read(&mut reader)
    }

    /// Get a reference to the constant pool of this class file.
    pub fn constant_pool(&self) -> &ConstantPool {
        &self.constant_pool
    }

    pub fn fields(&self) -> &Vec<FieldInfo> {
        &self.fields
    }

    pub fn methods(&self) -> &Vec<MethodInfo> {
        &self.methods
    }

    /// Get the name of the current class.
    pub fn class_name<'a>(&'a self) -> Result<Cow<'a, str>, DecodingError> {
        match self.constant_pool.get_class_name(self.this_class as usize) {
            Some(name) => Ok(name),
            None => Err(DecodingError::InvalidThisClass {
                index: self.this_class as usize,
                message: Some(format!(
                    "entry found: {:?}",
                    self.constant_pool.get(self.this_class as usize)
                )),
            }),
        }
    }

    /// Get the name of the super class.
    ///
    /// Returns `Ok(None)` if the super class is [java.lang.Object]. Otherwise, the super
    /// class name is returned.
    pub fn super_class_name<'a>(&'a self) -> Result<Option<Cow<'a, str>>, DecodingError> {
        if self.super_class == 0 {
            Ok(None)
        } else {
            match self.constant_pool.get_class_name(self.super_class as usize) {
                Some(name) => Ok(Some(name)),
                None => Err(DecodingError::InvalidSuperClass {
                    index: self.super_class as usize,
                    message: Some(format!(
                        "entry found: {:?}",
                        self.constant_pool.get(self.super_class as usize)
                    )),
                }),
            }
        }
    }

    /// Get the name of the super interfaces.
    pub fn super_interfaces_names<'a>(&'a self) -> Result<Vec<Cow<'a, str>>, DecodingError> {
        let mut names = Vec::new();
        for interface in &self.interfaces {
            match self.constant_pool.get_class_name(*interface as usize) {
                Some(name) => names.push(name),
                None => {
                    return Err(DecodingError::InvalidInterface {
                        index: *interface as usize,
                        message: None,
                    })
                }
            }
        }
        Ok(names)
    }

    /// Get the access flags of this class.
    pub fn access_flags(&self) -> FlagSet<ClassAccessFlags> {
        self.access_flags
    }
}

#[derive(BinRead, Debug, Clone)]
#[br(big)]
pub struct FieldInfo {
    /// Access flags denoting the permissions and properties of this field.
    #[br(map= |x: U2| FlagSet::<FieldAccessFlags>::new_truncated(x))]
    pub access_flags: FlagSet<FieldAccessFlags>,
    /// Unqualified name denoting the field.
    /// The index must point to a valid [crate::base::constant_pool::Utf8Info] in the constant pool.
    pub name_index: U2,
    /// Unqualified name denoting the field descriptor.
    /// The index must point to a valid [crate::base::constant_pool::Utf8Info] in the constant pool.
    pub descriptor_index: U2,
    // Attributes count
    attributes_count: U2,
    /// Attribute table of the field
    #[br(count=attributes_count)]
    pub attributes: Vec<AttributeInfo>,
}

#[derive(BinRead, Debug, Clone)]
#[br(big)]
pub struct MethodInfo {
    /// Access flags denoting the permissions and properties of this method.
    #[br(map= |x: U2| FlagSet::<MethodAccessFlags>::new_truncated(x))]
    pub access_flags: FlagSet<MethodAccessFlags>,
    /// Unqualified name denoting the method.
    /// The index must point to a valid [crate::base::constant_pool::Utf8Info] in the constant pool.
    pub name_index: U2,
    /// Unqualified name denoting the method descriptor.
    /// The index must point to a valid [crate::base::constant_pool::Utf8Info] in the constant pool.
    pub descriptor_index: U2,
    // Attributes count
    attributes_count: U2,
    /// Attribute table of the method
    #[br(count=attributes_count)]
    pub attributes: Vec<AttributeInfo>,
}

flags! {
    /// Access flags for classes, interfaces and modules.
    /// See [JVMS 4.1](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.1).
    #[derive(Collectable)]
    pub enum ClassAccessFlags: U2 {
        /// Declared public; may be accessed from outside its package.
        Public = 0x0001,
        /// Declared final; no subclasses allowed.
        Final = 0x0010,
        /// Treat superclass methods specially when invoked by the invokespecial
        /// instruction.
        Super = 0x0020,
        /// Is an interface, not a class.
        Interface = 0x0200,
        /// Declared abstract; must not be instantiated.
        Abstract = 0x0400,
        /// Declared synthetic; not present in the source code.
        Synthetic = 0x1000,
        /// Declared as an annotation interface.
        Annotation = 0x2000,
        /// Declared as an enum class.
        Enum = 0x4000,
        /// Module, not a class or interface.
        Module = 0x8000,
    }

    /// Access flags for fields.
    /// See [JVMS 4.5](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.5).
    #[derive(Collectable)]
    pub enum FieldAccessFlags: U2 {
        /// Declared public; may be accessed from outside its package.
        Public = 0x0001,
        /// Declared private; usable only within the defining class and other
        /// classes belonging to the same nest as the defining class.
        Private = 0x0002,
        /// Declared protected; may be accessed within subclasses.
        Protected = 0x0004,
        /// Declared static.
        Static = 0x0008,
        /// Declared final; never directly assigned to after object construction.
        Final = 0x0010,
        /// Declared volatile; cannot be cached.
        Volatile = 0x0040,
        /// Declared transient; not written or read by a persistent object manager.
        Transient = 0x0080,
        /// Declared synthetic; not present in the source code.
        Synthetic = 0x1000,
        /// Declared as an element of an enum.
        Enum = 0x4000,
    }

    /// Access flags for methods.
    /// See [JVMS 4.6](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.6).
    #[derive(Collectable)]
    pub enum MethodAccessFlags: U2 {
        /// Declared public; may be accessed from outside its package.
        Public = 0x0001,
        /// Declared private; accessible only within the defining class
        /// and other classes belonging to the same nest as the defining class.
        Private = 0x0002,
        /// Declared protected; may be accessed within subclasses.
        Protected = 0x0004,
        /// Declared static.
        Static = 0x0008,
        /// Declared final; must not be overridden.
        Final = 0x0010,
        /// Declared synchronized; invocation is wrapped by a monitor use.
        Synchronized = 0x0020,
        /// A bridge method, generated by the compiler.
        Bridge = 0x0040,
        /// Declared with variable number of arguments.
        Varargs = 0x0080,
        /// Declared native; implemented in a language other than Java.
        Native = 0x0100,
        /// Declared abstract; no implementation is provided.
        Abstract = 0x0400,
        /// Declared strictfp; floating-point mode is FP-strict.
        /// Deprecated, since Java SE 17.
        Strict = 0x0800,
        /// Declared synthetic; not present in the source code.
        Synthetic = 0x1000,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_minimal_class() {
        let bytecode = include_bytes!("../../res/test/MinimalClass.class");
        let mut bytes = Cursor::new(bytecode);
        let classfile = ClassFile::read(&mut bytes).unwrap();
        assert_eq!(classfile.magic, 0xCAFEBABE);
        assert_eq!(classfile.minor_version, 0);
        assert_eq!(classfile.major_version, 65);
        assert_eq!(classfile.constant_pool_count, 18);
        assert_eq!(classfile.constant_pool.0.len(), 17);
        assert_eq!(
            classfile.access_flags,
            FlagSet::<ClassAccessFlags>::new_truncated(0x0020)
        );
        assert_eq!(classfile.this_class, 7);
        assert_eq!(classfile.super_class, 2);
        assert_eq!(classfile.interfaces_count, 0);
        assert_eq!(classfile.interfaces.len(), 0);
        assert_eq!(classfile.fields_count, 1);
        assert_eq!(classfile.fields.len(), 1);
        let field = &classfile.fields[0];
        assert_eq!(
            field.access_flags,
            FlagSet::<FieldAccessFlags>::new_truncated(0x0018)
        );
        assert_eq!(field.name_index, 9);
        assert_eq!(field.descriptor_index, 10);
        assert_eq!(classfile.methods_count, 2);
        assert_eq!(classfile.methods.len(), 2);
        let init_method = &classfile.methods[0];
        assert_eq!(
            init_method.access_flags,
            FlagSet::<MethodAccessFlags>::new_truncated(0)
        );
        assert_eq!(init_method.name_index, 5);
        assert_eq!(init_method.descriptor_index, 6);
        assert_eq!(init_method.attributes_count, 1);
        assert_eq!(init_method.attributes.len(), 1);
        let init_code_attribute = &init_method.attributes[0];
        assert_eq!(init_code_attribute.attribute_name_index, 13);
        assert_eq!(init_code_attribute.attribute_length, 29);
        assert_eq!(init_code_attribute.info.len(), 29);
        let main_method = &classfile.methods[1];
        assert_eq!(
            main_method.access_flags,
            FlagSet::<MethodAccessFlags>::new_truncated(0x0009)
        );
        assert_eq!(main_method.name_index, 15);
        assert_eq!(main_method.descriptor_index, 6);
        assert_eq!(main_method.attributes_count, 1);
        assert_eq!(main_method.attributes.len(), 1);
        let main_code_attribute = &main_method.attributes[0];
        assert_eq!(main_code_attribute.attribute_name_index, 13);
        assert_eq!(main_code_attribute.attribute_length, 39);
        assert_eq!(main_code_attribute.info.len(), 39);
        assert_eq!(classfile.attributes_count, 1);
        assert_eq!(classfile.attributes.len(), 1);
        let source_file_attribute = &classfile.attributes[0];
        assert_eq!(source_file_attribute.attribute_name_index, 16);
        assert_eq!(source_file_attribute.attribute_length, 2);
        assert_eq!(source_file_attribute.info.len(), 2);
    }
}
