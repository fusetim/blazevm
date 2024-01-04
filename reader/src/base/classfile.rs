use flagset::{FlagSet, flags};
use binrw::{BinRead, BinReaderExt, binrw};
use super::{U1, U2, U4, ConstantPool};

/// Model of a Class Info
///
/// The classfile structure represents the entire class file read.
/// Note: One class or module is always represented by one class file.
#[derive(BinRead)]
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
    #[br(args(constant_pool_count))]
    constant_pool: ConstantPool,
    /// Access flags
    /// Flags indicating access permissions to and properties of this class,
    /// interface or module.
    access_flags: AccessFlags,
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

#[derive(BinRead)]
#[br(big)]
pub struct FieldInfo {
    /// Access flags denoting the permissions and properties of this field.
    access_flags: AccessFlags,
    /// Unqualified name denoting the field.
    /// The index must point to a valid [crate::base::constant_pool::Utf8Info] in the constant pool.
    name_index: U2,
    /// Unqualified name denoting the field descriptor.
    /// The index must point to a valid [crate::base::constant_pool::Utf8Info] in the constant pool.
    descriptor_index: U2,
    // Attributes count
    attributes_count: U2,
    /// Attribute table of the field
    #[br(count=attributes_count)]
    attributes: Vec<AttributeInfo>,
}

#[derive(BinRead)]
#[br(big)]
pub struct MethodInfo {
    /// Access flags denoting the permissions and properties of this method.
    access_flags: AccessFlags,
    /// Unqualified name denoting the method.
    /// The index must point to a valid [crate::base::constant_pool::Utf8Info] in the constant pool.
    name_index: U2,
    /// Unqualified name denoting the method descriptor.
    /// The index must point to a valid [crate::base::constant_pool::Utf8Info] in the constant pool.
    descriptor_index: U2,
    // Attributes count
    attributes_count: U2,
    /// Attribute table of the method
    #[br(count=attributes_count)]
    attributes: Vec<AttributeInfo>,
}

#[derive(BinRead)]
#[br(big)]
pub struct AttributeInfo {
    /// Unqualified name denoting the attribute.
    /// The index must point to a valid [crate::base::constant_pool::Utf8Info] in the constant pool.
    attribute_name_index: U2,
    // Info length
    attribute_length: U4,
    /// Variable-length info
    #[br(count=attribute_length)]
    info: Vec<U1>,
}

flags! {
    /// Access flags
    /// Flags indicating access permissions to and properties of this class,
    /// interface, module, fields or methods.
    #[derive(BinRead)]
    #[br(big, repr=U2)]
    enum AccessFlags: U2 {
        /// Declared public, it may be accessed from outside its package.
        Public = 0x0001,
        /// Declared private, accessible only within the defining class.
        /// Only applicable to methods and fields.
        Private = 0x0002,
        /// Declared protected, may be accessed within subclasses.
        /// Only applicable to methods and fields.
        Protected = 0x0004,
        /// Declared static.
        /// Only applicable to methods and fields.
        Static = 0x0008,
        /// Declared final, no subclasses allowed.
        Final = 0x0010,
        /// Declared synchronized, invocation is wrapped by a monitor use.
        /// Only applicable to methods.
        Synchronized = 0x0020,
        /// Volatile, cannot be cached.
        /// Only applicable to fields.
        Volatile = 0x0040,
        /// A bridge method, generated by the compiler.
        /// Only applicable to methods.
        Bridge = 0x0040,
        /// Declared with variable number of arguments.
        /// Only applicable to methods.
        Varargs = 0x0080,
        /// Declared native, implemented in a language other than Java.
        /// Only applicable to methods.
        Native = 0x0100,
        /// Special property, that treats superclass methods particularly
        /// when invoked by the invokespecial instruction.
        /// Only applicable to methods.
        Super = 0x0020,
        /// Is an interface.
        Interface = 0x0200,
        /// Declared abstract, it should not be instantiated.
        Abstract = 0x0400,
        /// Declared strictfp, floating-point mode is FP-strict.
        /// Only applicable to classfiles generated for Java SE between 1.2 and 16.
        /// Only applicable to methods and fields.
        /// We probably won't support this.
        Strict = 0x0800,
        /// Declared synthetic, not present in the source code.
        /// Only applicable to methods and fields.
        Synthetic = 0x1000,
        /// Declared as an annotation type.
        /// Only applicable to classes and interfaces.
        Annotation = 0x2000,
        /// Declared as an enum type.
        /// Only applicable to classes and interfaces.
        Enum = 0x4000,
        /// Is a module.
        /// Only applicable to modules (and it is the only access flag that should be triggered).
        Module = 0x8000,
    }
}