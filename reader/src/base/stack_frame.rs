use binrw::{binrw, BinRead, BinReaderExt};
use super::{U1, U2, U4};

/// Entry of the stack map table of a [StackMapTableAttribute].
///
/// Represents the state of the operand stack and local variables at a particular
/// point in the code array.
pub enum StackMapFrame {
    SameFrame(SameFrame),
    SameLocals1StackItemFrame(SameLocals1StackItemFrame),
    SameLocals1StackItemFrameExtended(SameLocals1StackItemFrameExtended),
    ChopFrame(ChopFrame),
    SameFrameExtended(SameFrameExtended),
    AppendFrame(AppendFrame),
    FullFrame(FullFrame),
}


/// This stack frame has exactly the same locals as the previous stack frame,
/// and the number of stack items is zero.
pub struct SameFrame {
    /// The offset_delta is the value of the frame_type item (0-63).
    pub offset_delta: U1,
}

/// This stack frame has exactly the same locals as the previous stack frame,
/// and the number of stack items is 1.
pub struct SameLocals1StackItemFrame{
    /// The offset_delta is the value of the frame_type item (64-127) minus 64.
    /// It is therefore an unsigned value between 0 and 63.
    pub offset_delta: U1,
    /// The verification type info of the stack item.
    pub stack: VerificationTypeInfo,
}

/// This stack frame indicates that the frame has exactly the same locals as the
/// previous stack frame and that the number of stack items is 1.
pub struct SameLocals1StackItemFrameExtended{
    // Frame type is 247.
    /// The value of the offset_delta item.
    pub offset_delta: U2,
    /// The verification type info of the stack item.
    pub stack: VerificationTypeInfo,
}

/// This stack frame indicates that the frame has exactly the same locals as the
/// previous stack frame but the last k locals variables are absent.
/// The operand stack is empty.
pub struct ChopFrame{
    /// The number k of absent locals.
    /// The value of k is given by the formula 251-frame_type, where frame_type is in range [248,250].
    pub k: U1,
    /// The value of the offset_delta item.
    pub offset_delta: U2,
}

/// This stack frame indicates that the frame has exactly the same locals as the
/// previous stack frame and that the number of stack items is zero.
pub struct SameFrameExtended{
    // Frame type is 251.

    /// The value of the offset_delta item.
    pub offset_delta: U2,
}

/// This stack frame indicates that the frame has exactly the same locals as the
/// previous stack frame but with k additional locals.
/// The operand stack is empty.
pub struct AppendFrame{
    // Frame type is in range [252-254].

    /// The number k of additional locals.
    /// The value of k is given by the formula frame_type-251, where frame_type is in range [252-254].
    pub k: U1,
    /// The value of the offset_delta item.
    pub offset_delta: U2,
    /// The verification type info of the k additional locals.
    pub locals: Vec<VerificationTypeInfo>,
}

/// This stack frame is fully specified.
pub struct FullFrame{
    // Frame type is 255.

    /// The value of the offset_delta item.
    pub offset_delta: U2,
    /// The number of local variables in the local variable array.
    pub number_of_locals: U2,
    /// The verification type info of the local variables.
    pub locals: Vec<VerificationTypeInfo>,
    /// The number of stack items in the operand stack.
    pub number_of_stack_items: U2,
    /// The verification type info of the stack items.
    pub stack: Vec<VerificationTypeInfo>,
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
    ObjectVariableInfo { cpool_index: U2 },
    #[br(magic = 8u8)]
    UninitializedVariableInfo { offset: U2 },
}
