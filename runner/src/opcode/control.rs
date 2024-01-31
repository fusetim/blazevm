use super::LookupSwitch;
use super::TableSwitch;
use super::{InstructionError, InstructionSuccess};
use crate::thread::Slot;
use crate::thread::Thread;
use crate::xreturn;

/// `goto` jumps to another instruction.
pub fn goto(thread: &mut Thread, offset: i16) -> Result<InstructionSuccess, InstructionError> {
    Ok(InstructionSuccess::JumpRelative(offset as isize))
}

/// `goto_w` (wide variant) jumps to another instruction.
pub fn goto_w(thread: &mut Thread, offset: i32) -> Result<InstructionSuccess, InstructionError> {
    Ok(InstructionSuccess::JumpRelative(offset as isize))
}

/// `jsr` pushes the address of the next instruction onto the stack and jumps to another instruction.
///
/// The address of the next instruction is pushed onto the stack as a return address, 32-bit value.
pub fn jsr(thread: &mut Thread, offset: i16) -> Result<InstructionSuccess, InstructionError> {
    let pc = thread.pc as u32;
    let frame = thread.current_frame_mut().unwrap();
    frame
        .operand_stack
        .push(Slot::ReturnAddress((pc + 3) as u32));
    Ok(InstructionSuccess::JumpRelative(offset as isize))
}

/// `jsr_w` (wide variant) pushes the address of the next instruction onto the
/// stack and jumps to another instruction.
///
/// The address of the next instruction is pushed onto the stack as a return address, 32-bit value.
pub fn jsr_w(thread: &mut Thread, offset: i32) -> Result<InstructionSuccess, InstructionError> {
    let pc = thread.pc as u32;
    let frame = thread.current_frame_mut().unwrap();
    frame
        .operand_stack
        .push(Slot::ReturnAddress((pc + 5) as u32));
    Ok(InstructionSuccess::JumpRelative(offset as isize))
}

/// `ret` returns from a subroutine.
///
/// The index is an unsigned byte that must be an index into the local variable array of the current frame.
pub fn ret(thread: &mut Thread, index: u8) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let Slot::ReturnAddress(address) = frame.local_variables[index as usize] else {
        return Err(InstructionError::InvalidState {
            context: format!("Expected return address at index {}", index),
        });
    };
    Ok(InstructionSuccess::JumpAbsolute(address as usize))
}

/// `tableswitch` accesses jump table by index and jumps.
pub fn tableswitch(
    thread: &mut Thread,
    table: &TableSwitch,
) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let index = frame.operand_stack.pop().unwrap();
    let offset = match index {
        Slot::Int(index) => {
            if index < table.low || index > table.high {
                table.default
            } else {
                table.jump_offsets[(index - table.low) as usize]
            }
        }
        _ => {
            return Err(InstructionError::InvalidState {
                context: "Expected int on the operand stack".into(),
            })
        }
    };
    Ok(InstructionSuccess::JumpRelative(offset as isize))
}

/// `lookupswitch` accesses jump table by key match and jumps.
pub fn lookupswitch(
    thread: &mut Thread,
    table: &LookupSwitch,
) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let key = frame.operand_stack.pop().unwrap();
    let offset = match key {
        Slot::Int(key) => {
            if let Ok(index) = table.match_offsets.binary_search_by_key(&key, |(k, _)| *k) {
                table.match_offsets[index].1
            } else {
                table.default
            }
        }
        _ => {
            return Err(InstructionError::InvalidState {
                context: "Expected int on the operand stack".into(),
            })
        }
    };
    Ok(InstructionSuccess::JumpRelative(offset as isize))
}

/// `return` returns void from a method.
pub fn vreturn(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    thread.pop_frame();
    // TODO: implement monitor strategy for synchronized methods
    if let Some(frame) = thread.current_frame_mut() {
        let Some(Slot::InvokationReturnAddress(pc)) = frame.operand_stack.pop() else {
            return Err(InstructionError::InvalidState {
                context: "Expected invokation return address on the operand stack".into(),
            });
        };
        Ok(InstructionSuccess::FrameChange(pc as usize))
    } else {
        Ok(InstructionSuccess::Completed)
    }
}

// TODO: ireturn actually checks the method type to cast properly the returned value to the correct type
// (bool, char, byte, short, int)
xreturn!(ireturn, Int);
xreturn!(lreturn, Long);
xreturn!(freturn, Float);
xreturn!(dreturn, Double);
// TODO: implement areturn

mod macros {
    #[macro_export]
    macro_rules! xreturn {
        ($name:ident, $ty:ident) => {
            /// Return a value from a method.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let prev_frame = thread.pop_frame().unwrap();
                // TODO: implement monitor strategy for synchronized methods
                if let Some(Slot::$ty(value)) = prev_frame.operand_stack.last() {
                    let frame = thread.current_frame_mut().unwrap();
                    let Some(Slot::InvokationReturnAddress(pc)) = frame.operand_stack.pop() else {
                        return Err(InstructionError::InvalidState {
                            context: "Expected invokation return address on the operand stack"
                                .into(),
                        });
                    };
                    frame.operand_stack.push(Slot::$ty(*value));
                    Ok(InstructionSuccess::FrameChange(pc as usize))
                } else {
                    return Err(InstructionError::InvalidState {
                        context: format!("Expected {:?} on the operand stack", stringify!($ty)),
                    });
                }
            }
        };
    }
}
