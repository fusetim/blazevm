use crate::thread::{Slot, Thread};

use super::{InstructionError, InstructionSuccess};


/// `ifnull` - Branch if reference is null
pub fn ifnull(thread: &mut Thread, offset: i16) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let value = frame.operand_stack.pop().unwrap();
    match value {
        Slot::UndefinedReference => Ok(InstructionSuccess::JumpRelative(offset as isize)),
        _ => Ok(InstructionSuccess::Next(3)),
    }
}

/// `ifnonnull` - Branch if reference is not null
pub fn ifnonnull(thread: &mut Thread, offset: i16) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let value = frame.operand_stack.pop().unwrap();
    match value {
        Slot::UndefinedReference => Ok(InstructionSuccess::Next(3)),
        _ => Ok(InstructionSuccess::JumpRelative(offset as isize)),
    }
}