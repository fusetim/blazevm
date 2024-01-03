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
    Entry(ConstantPoolInfo),
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
    magic: U1,
    minor_version: U2,
    major_version: U2,
    constant_pool_count: U2,
    constant_pool: ConstantPool,
    access_flags: U2,
    this_class: U2,
    super_class: U2,
    interfaces_count: U2,
    interfaces: Vec<U2>,
    fields_count: U2,
    fields: Vec<FieldInfo>,
    methods_count: U2,
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
