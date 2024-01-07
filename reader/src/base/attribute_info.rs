use super::{ConstantPool, U1, U2, U4, StackMapFrame, stack_frame::parse_stack_map_frame};
use binrw::{binrw, BinRead, BinReaderExt, BinResult};

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

/// Attribute ConstantValue, a member of [AttributeInfo].
///
/// Represents the value (by reference) of a constant field.
#[derive(BinRead)]
#[br(big)]
pub struct ConstantValueAttribute {
    /// Index of the constant in the constant pool.
    /// The constant must be of the same type as the field.
    constant_value_index: U2,
}

/// Attribute Code, a member of [AttributeInfo].
///
/// Represents the body of a method.
/// It contains the bytecode, the exception table, and the attributes,
/// and some auxiliary information.
#[derive(BinRead)]
#[br(big)]
pub struct CodeAttribute {
    /// The max length of the operand stack of this method.
    max_stack: U2,
    /// The maximum number of local variables in the local variable array allocated
    /// upon invocation of this method.
    max_locals: U2,
    /// The number of bytes in the bytecode array.
    code_length: U4,
    /// The bytecode array.
    #[br(count=code_length)]
    code: Vec<U1>,
    /// The number of entries in the exception table.
    exception_table_length: U2,
    /// The exception table.
    #[br(count=exception_table_length)]
    exception_table: Vec<ExceptionTableEntry>,
    /// The number of attributes in the attributes table.
    attributes_count: U2,
    /// The attributes table.
    #[br(count=attributes_count)]
    attributes: Vec<AttributeInfo>,
}

/// Entry of the exception table of a [CodeAttribute].
#[derive(BinRead)]
#[br(big)]
pub struct ExceptionTableEntry {
    /// Indicates the start of the code range where the exception handler is active.
    start_pc: U2,
    /// Indicates the end of the code range where the exception handler is active.
    end_pc: U2,
    /// Indicates the first instruction of the exception handler to run.
    handler_pc: U2,
    /// Index of a [ClassInfo] in the constant pool.
    ///
    /// If non-zero, it represents the Exception class of exception handled by the catch clause.
    /// If zero, it represents a catch clause that handles all types of exceptions.
    catch_type: U2,
}

/// Atribute StackMapTable, a member of [AttributeInfo].
///
/// Represents the stack map table of a method.
#[derive(BinRead)]
#[br(big)]
pub struct StackMapTableAttribute {
    /// The number of entries in the stack map table.
    number_of_entries: U2,
    /// The stack map table.
    #[br(parse_with=parse_stack_map_entries, args(number_of_entries as usize))]
    entries: Vec<StackMapFrame>,
}

#[binrw::parser(reader, endian)]
fn parse_stack_map_entries(count: usize) -> BinResult<Vec<StackMapFrame>> {
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let entry: StackMapFrame = parse_stack_map_frame(reader, endian, ())?;
        entries.push(entry);
    }
    Ok(entries)
}

/// Attribute BootstrapMethods, a member of [AttributeInfo].
///
/// This attribute records bootstrap methods used by dynamic instructions.
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.23>
#[derive(BinRead)]
#[br(big)]
pub struct BootstrapMethodsAttribute {
    /// The number of bootstrap methods in the bootstrap_methods array.
    pub num_bootstrap_methods: U2,

    /// The bootstrap methods.
    #[br(count=num_bootstrap_methods)]
    pub bootstrap_methods: Vec<BootstrapMethod>,
}

/// A bootstrap method, a member of [BootstrapMethodsAttribute].
///
/// This structure represents a bootstrap method, which is a method that is invoked
/// during the invocation of a dynamic instruction.
/// It invokes a method to compute the value of a number of static arguments.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.23>
#[derive(BinRead)]
#[br(big)]
pub struct BootstrapMethod {
    /// A reference to a [MethodHandleInfo] in the constant pool.
    pub bootstrap_method_ref: U2,
    /// The number of items in the bootstrap_arguments array.
    pub num_bootstrap_arguments: U2,
    /// The bootstrap **static** arguments, referenced by their indices in the constant pool.
    #[br(count=num_bootstrap_arguments)]
    pub bootstrap_arguments: Vec<U2>,
}

/// Attribute NestHost, a member of [AttributeInfo].
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.28>
#[derive(BinRead)]
#[br(big)]
pub struct NestHostAttribute {
    /// A reference to a [ClassInfo] in the constant pool.
    ///
    /// The class/interface is the nest host of the current class/interface.
    pub host_class_index: U2,
}

/// Attribute NestMembers, a member of [AttributeInfo].
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.29>
#[derive(BinRead)]
#[br(big)]
pub struct NestMembersAttribute {
    /// The number of entries in the classes array.
    pub num_classes: U2,
    /// The classes/interfaces that are members of the nest to which the current class/interface belongs.
    /// Each entry is a reference to a [ClassInfo] in the constant pool.
    #[br(count=num_classes)]
    pub classes: Vec<U2>,
}

/// Attribute PermittedSubclasses, a member of [AttributeInfo].
///
/// This attribute records the classes that are permitted to extend the current class.
///
/// Note: For final classes (cf [ClassAccessFlags::FINAL](super::classfile::ClassAccessFlags)), this 
/// attribute MUST exist and MUST be empty.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.31>
#[derive(BinRead)]
#[br(big)]
pub struct PermittedSubclassesAttribute {

    pub num_classes: U2,
    #[br(count=num_classes)]
    pub classes: Vec<U2>,
}
