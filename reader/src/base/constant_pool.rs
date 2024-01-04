use super::{U1, U2, U4};

/// Model of the Constant Pool
///
/// The constant pool is a tables representing the differents constants used later
/// on in the class file. Each entry might represent the name of a class, a method
/// or a field or litteral constants such as strings, integers, floats, etc.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.4>
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
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.4>
pub enum ConstantPoolInfo {
    /// ClassInfo entry, see [ClassInfo].
    ClassInfo(ClassInfo),
    FieldRefInfo(FieldRefInfo),
    MethodRefInfo(MethodRefInfo),
    InterfaceMethodRefInfo(InterfaceMethodRefInfo),
    StringInfo(StringInfo),
    IntegerInfo(IntegerInfo),
    // FloatInfo(FloatInfo),
    // LongInfo(LongInfo),
    // DoubleInfo(DoubleInfo),
    NameAndTypeInfo(NameAndTypeInfo),
    /// UTF8Info entry, see [Utf8Info].
    Utf8Info(Utf8Info),
    // MethodHandleInfo(MethodHandleInfo),
    // MethodTypeInfo(MethodTypeInfo),
    // DynamicInfo(DynamicInfo),
    // InvokeDynamicInfo(InvokeDynamicInfo),
    // ModuleInfo(ModuleInfo),
    // PackageInfo(PackageInfo),
}

/// ClassInfo is a [ConstantPool] entry.
///
/// It gives the index in the [ConstantPool] of a [Utf8Info] entry,
/// describing a valid binary name for the current class/interface/module.
pub struct ClassInfo {
    name_index: U2,
}

/// Utf8Info is a [ConstantPool] entry.
pub struct Utf8Info {
    // tag: U1,
    // length is the byte-length of the bytes fields, the resulting string might
    // be shorter.
    // length: U2,
    /// A CESU-8 encoded string
    bytes: Vec<U1>,
}

/// FieldRefInfo is a [ConstantPool] entry.
pub struct FieldRefInfo {
    // tag: U1,
    /// [ClassInfo] reference in the [ConstantPool].
    /// Such class/interface has the field as a member.
    class_index: U2,
    /// [NameAndTypeInfo] reference in the [ConstantPool].
    /// It identifies the name and descriptor of the field.
    ///
    /// NOTE: it should be checked that the descriptor is indeed a field descriptor.
    name_and_type_index: U2,
}

/// MethodRefInfo is a [ConstantPool] entry.
pub struct MethodRefInfo {
    // tag: U1,
    /// [ClassInfo] reference in the [ConstantPool].
    /// Such class has the method as a member.
    class_index: U2,
    /// [NameAndTypeInfo] reference in the [ConstantPool].
    /// It identifies the name and descriptor of the method.
    ///
    /// NOTE: it should be checked that the descriptor is indeed a method descriptor.
    name_and_type_index: U2,
}

/// InterfaceMethodRefInfo is a [ConstantPool] entry.
pub struct InterfaceMethodRefInfo {
    // tag: U1,
    /// [ClassInfo] reference in the [ConstantPool].
    /// Such interface has the method as a member.
    class_index: U2,
    /// [NameAndTypeInfo] reference in the [ConstantPool].
    /// It identifies the name and descriptor of the method.
    ///
    /// NOTE: it should be checked that the descriptor is indeed a method descriptor.
    name_and_type_index: U2,
}

/// StringInfo is a [ConstantPool] entry.
pub struct StringInfo {
    // tag: U1,
    /// A reference to a [Utf8Info] part of the [ConstantPool].
    /// This is the encoded representation of the string.
    string_index: U2,
}

/// IntegerInfo is a [ConstantPool] entry.
pub struct IntegerInfo {
    // tag: U1,
    /// Representation of the constant in big-endian order.
    bytes: U4,
}

/// NameAndTypeInfo is a [ConstantPool] entry.
pub struct NameAndTypeInfo {
    // tag: U1,
    /// Reference to a [Utf8Info] in the [ConstantPool].
    /// The name must be a valid unqualified name denoting a field
    /// or a method, OR, the special method name `<init>`.
    name_index: U2,
    /// Reference to a [Utf8Info] in the [ConstantPool].
    /// The descriptor must be a valid field or method descriptor.
    descriptor_index: U2,
}