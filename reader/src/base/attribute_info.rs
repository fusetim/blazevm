use binrw::{BinRead, BinReaderExt, binrw};
use super::{U1, U2, U4, ConstantPool};

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
// #[derive(BinRead)]
// #[br(big)]
pub struct StackMapTableAttribute {
    /// The number of entries in the stack map table.
    number_of_entries: U2,
    /// The stack map table.
    entries: Vec<StackMapFrame>,
}

/// Entry of the stack map table of a [StackMapTableAttribute].
///
/// Represents the state of the operand stack and local variables at a particular
/// point in the code array.
pub enum StackMapFrame {
    SameFrame {
        frame_type: U1,
    },
    SameLocals1StackItemFrame {
        frame_type: U1,
        stack: VerificationTypeInfo,
    },
    SameLocals1StackItemFrameExtended {
        frame_type: U1,
        offset_delta: U2,
        stack: VerificationTypeInfo,
    },
    ChopFrame {
        frame_type: U1,
        offset_delta: U2,
    },
    SameFrameExtended {
        frame_type: U1,
        offset_delta: U2,
    },
    AppendFrame {
        frame_type: U1,
        offset_delta: U2,
        locals: Vec<VerificationTypeInfo>,
    },
    FullFrame {
        frame_type: U1,
        offset_delta: U2,
        number_of_locals: U2,
        locals: Vec<VerificationTypeInfo>,
        number_of_stack_items: U2,
        stack: Vec<VerificationTypeInfo>,
    },
}

/// Verification type info, a member of [StackMapFrame].
///
/// Represents the type of a local variable or an operand stack entry.
#[derive(BinRead)]
#[br(big)]
pub enum VerificationTypeInfo {
    #[br(magic = 0u8)]
    TopVariableInfo,
    #[br(magic = 1u8)]
    IntegerVariableInfo,
    #[br(magic = 2u8)]
    FloatVariableInfo,
    #[br(magic = 3u8)]
    DoubleVariableInfo,
    #[br(magic = 4u8)]
    LongVariableInfo,
    #[br(magic = 5u8)]
    NullVariableInfo,
    #[br(magic = 6u8)]
    UninitializedThisVariableInfo,
    #[br(magic = 7u8)]
    ObjectVariableInfo {
        cpool_index: U2,
    },
    #[br(magic = 8u8)]
    UninitializedVariableInfo {
        offset: U2,
    },
}