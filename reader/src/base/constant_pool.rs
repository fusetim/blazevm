use std::borrow::Cow;

use binrw::{BinRead, BinReaderExt, BinResult};
use cesu8::from_java_cesu8;
use super::{U1, U2, U4};

/// Model of the Constant Pool
///
/// The constant pool is a tables representing the differents constants used later
/// on in the class file. Each entry might represent the name of a class, a method
/// or a field or litteral constants such as strings, integers, floats, etc.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.4>
#[derive(BinRead, Debug)]
#[br(big, import(count: U2))]
pub struct ConstantPool (#[br(parse_with = parse_constant_pool, args(count))] pub Vec<ConstantPoolEntry>);

/// Model of a Constant Pool Entry
///
/// Each entry might be a real entry or a tombstone. The tombstone is used to
/// mark the end of a long entry (such as a long or a double) in the constant
/// pool. Indeed those long entries take two slots in the constant pool, and
/// therefore to keep the same indexing, we need to mark the second slot as
/// a tombstone.
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(BinRead, Debug)]
#[br(big)]
pub struct ClassInfo {
    name_index: U2,
}

/// Utf8Info is a [ConstantPool] entry.
#[derive(BinRead, Debug)]
#[br(big)]
pub struct Utf8Info {
    // tag: U1,
    // length is the byte-length of the bytes fields, the resulting string might
    // be shorter.
    length: U2,
    /// A CESU-8 encoded string
    #[br(count=length)]
    bytes: Vec<U1>,
}

impl Utf8Info {
    /// Convert the internal Java CESU-8 encoded string to a Rust string.
    ///
    /// If the conversion fails, None is returned.
    pub fn to_string<'a>(&'a self) -> Option<Cow<'a, str>> {
        from_java_cesu8(self.bytes.as_slice()).ok()
    }
}

/// FieldRefInfo is a [ConstantPool] entry.
#[derive(BinRead, Debug)]
#[br(big)]
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
#[derive(BinRead, Debug)]
#[br(big)]
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
#[derive(BinRead, Debug)]
#[br(big)]
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
#[derive(BinRead, Debug)]
#[br(big)]
pub struct StringInfo {
    // tag: U1,
    /// A reference to a [Utf8Info] part of the [ConstantPool].
    /// This is the encoded representation of the string.
    string_index: U2,
}

/// IntegerInfo is a [ConstantPool] entry.
#[derive(BinRead, Debug)]
#[br(big)]
pub struct IntegerInfo {
    // tag: U1,
    /// Representation of the constant in big-endian order.
    bytes: U4,
}

/// NameAndTypeInfo is a [ConstantPool] entry.
#[derive(BinRead, Debug)]
#[br(big)]
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

/// Parser for the [ConstantPool].
#[binrw::parser(reader, endian)]
fn parse_constant_pool(count: U2) -> BinResult<Vec<ConstantPoolEntry>> {
    let count = count as usize;
    let mut entries = Vec::with_capacity(count);
    let mut i = 0;
    while i < count {
        let tag = U1::read_be(reader)?;
        let (entry, tombstone) = match tag {
            1 => (ConstantPoolEntry::Entry(ConstantPoolInfo::Utf8Info(Utf8Info::read(reader)?)), false),
            3 => (ConstantPoolEntry::Entry(ConstantPoolInfo::IntegerInfo(IntegerInfo::read(reader)?)), false),
            7 => (ConstantPoolEntry::Entry(ConstantPoolInfo::ClassInfo(ClassInfo::read(reader)?)), false),
            8 => (ConstantPoolEntry::Entry(ConstantPoolInfo::StringInfo(StringInfo::read(reader)?)), false),
            9 => (ConstantPoolEntry::Entry(ConstantPoolInfo::FieldRefInfo(FieldRefInfo::read(reader)?)), false),
            10 => (ConstantPoolEntry::Entry(ConstantPoolInfo::MethodRefInfo(MethodRefInfo::read(reader)?)), false),
            11 => (ConstantPoolEntry::Entry(ConstantPoolInfo::InterfaceMethodRefInfo(InterfaceMethodRefInfo::read(reader)?)), false),
            12 => (ConstantPoolEntry::Entry(ConstantPoolInfo::NameAndTypeInfo(NameAndTypeInfo::read(reader)?)), false),
            x => unimplemented!("Constant pool tag {} not implemented", x),
        };
        entries.push(dbg!(entry));
        i += 1;
        if tombstone {
            entries.push(ConstantPoolEntry::Tombstone);
            i += 1;
        }
    }
    Ok(entries)
}


#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_utf8_info() {
        let data = [0x00, 0x0B, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xE2, 0x9C, 0x8B, 0xF0, 0x9F, 0x98];
        let mut reader = Cursor::new(&data);
        let info = Utf8Info::read(&mut reader).unwrap();
        assert_eq!(info.length, 11);
        assert_eq!(info.bytes, vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xE2, 0x9C, 0x8B, 0xF0, 0x9F, 0x98]);
    }

    #[test]
    fn load_utf8i_in_constant_pool() {
        let data = [0x01, 0x00, 0x0B, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xE2, 0x9C, 0x8B, 0xF0, 0x9F, 0x98];
        let mut reader = Cursor::new(&data);
        let pool = ConstantPool::read_args(&mut reader, (1,)).unwrap();
        assert!(pool.0.len() == 1);
        assert!(matches!(pool.0[0], ConstantPoolEntry::Entry(ConstantPoolInfo::Utf8Info(_))));
    }
}