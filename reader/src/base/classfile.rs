use flagset::{FlagSet, flags};

pub type U1 = u8;
pub type U2 = u16;
pub type U4 = u32;

/// Model of the Constant Pool
///
/// The constant pool is a tables representing the differents constants used later
/// on in the class file. Each entry might represent the name of a class, a method
/// or a field or litteral constants such as strings, integers, floats, etc.
///
/// Ref: https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.4
pub type ConstantPool = Vec<ConstantPoolEntry>;

/// Model of a Constant Pool Entry
///
/// Each entry might be a real entry or a tombstone. The tombstone is used to
/// mark the end of a long entry (such as a long or a double) in the constant
/// pool. Indeed those long entries take two slots in the constant pool, and
/// therefore to keep the same indexing, we need to mark the second slot as
/// a tombstone.
pub enum ConstantPoolEntry {
    /// A real entry in the constant pool
    Entry(ConstantPoolInfo),
    /// Special marker, to keep the indices of the constant pool entries
    /// consistent with the classfile specification.
    Tombstone,
}

/// Model of a Constant Pool Info
///
/// Each entry in the constant pool is a constant pool info. The constant pool
/// info is defined in the classfile by a tag, which is a u1, and the content
/// that is of variable size, depending on the tag.
///
/// We don't need to model the structure actually used in the classfile, therefore
/// we can use an enum to represent the different types of constant pool info, and
/// use serialization and deserialization magic to convert from the classfile
/// representation to ours.
///
/// Ref: https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.4
pub enum ConstantPoolInfo {
    // ClassInfo(ClassInfo),
    // FieldRefInfo(FieldRefInfo),
    // MethodRefInfo(MethodRefInfo),
    // InterfaceMethodRefInfo(InterfaceMethodRefInfo),
    // StringInfo(StringInfo),
    // IntegerInfo(IntegerInfo),
    // FloatInfo(FloatInfo),
    // LongInfo(LongInfo),
    // DoubleInfo(DoubleInfo),
    // NameAndTypeInfo(NameAndTypeInfo),
    // Utf8Info(Utf8Info),
    // MethodHandleInfo(MethodHandleInfo),
    // MethodTypeInfo(MethodTypeInfo),
    // DynamicInfo(DynamicInfo),
    // InvokeDynamicInfo(InvokeDynamicInfo),
    // ModuleInfo(ModuleInfo),
    // PackageInfo(PackageInfo),
}

/// Model of a Class Info
///
/// The classfile structure represents the entire class file read.
/// Note: One class or module is always represented by one class file.
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
    /// Constant pool count
    /// The number of entries in the constant pool table plus one.
    /// This is because the constant pool is indexed from 1 to n-1.
    constant_pool_count: U2,
    /// Constant pool
    constant_pool: ConstantPool,
    /// Access flags
    /// Flags indicating access permissions to and properties of this class,
    /// interface or module.
    access_flags: AccessFlags,
    /// Pointer to the [ClassInfo] of the current class/interface in the constant pool.
    this_class: U2,
    /// Pointer to the [ClassInfo] of the super class/interface in the constant pool.
    ///
    /// For a class, this is the super class of the current class. 0 if the class is
    /// [java.lang.Object].
    /// For an interface, points to the [ClassInfo] of [java.lang.Object].
    super_class: U2,
    /// Interfaces count
    /// The number of direct super interfaces of this class or interface type.
    interfaces_count: U2,
    /// The direct super interfaces of this class or interface type.
    /// Each entry must be a valid index into the constant pool table.
    /// The order of the interfaces is significant, and should be preserved.
    interfaces: Vec<U2>,
    /// Fields count
    /// The number of fields of this class or interface type.
    fields_count: U2,
    /// The fields' index into the constant pool table.
    /// It only contains the fields defined by this class or interface, and not
    /// those inherited from super classes or interfaces.
    fields: Vec<FieldInfo>,
    /// Methods count
    /// The number of methods of this class or interface type.
    methods_count: U2,
    /// 
    methods: Vec<MethodInfo>,
    attributes_count: U2,
    attributes: Vec<AttributeInfo>,
}

pub struct FieldInfo {
    access_flags: U2,
    name_index: U2,
    descriptor_index: U2,
    attributes_count: U2,
    attributes: Vec<AttributeInfo>,
}

pub struct MethodInfo {
    access_flags: U2,
    name_index: U2,
    descriptor_index: U2,
    attributes_count: U2,
    attributes: Vec<AttributeInfo>,
}

pub struct AttributeInfo {
    attribute_name_index: U2,
    attribute_length: U4,
    info: Vec<U1>,
}

flags! {
    /// Access flags
    /// Flags indicating access permissions to and properties of this class,
    /// interface, module, fields or methods.
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
        /// Only applicable to classes and interfaces.
        Module = 0x8000,
    }
}