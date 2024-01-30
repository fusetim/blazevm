use super::InstructionError;
use super::LookupSwitch;
use super::TableSwitch;
use crate::thread::Slot;
use crate::thread::Thread;
use crate::xreturn;

/// `goto` jumps to another instruction.
pub fn goto(thread: &mut Thread, offset: i16) -> Result<(), InstructionError> {
    thread.pc = (thread.pc as i32 + offset as i32) as usize;
    Ok(())
}

/// `goto_w` (wide variant) jumps to another instruction.
pub fn goto_w(thread: &mut Thread, offset: i32) -> Result<(), InstructionError> {
    thread.pc = (thread.pc as i32 + offset) as usize;
    Ok(())
}

/// `jsr` pushes the address of the next instruction onto the stack and jumps to another instruction.
///
/// The address of the next instruction is pushed onto the stack as a return address, 32-bit value.
pub fn jsr(thread: &mut Thread, offset: i16) -> Result<(), InstructionError> {
    let pc = thread.pc as u32;
    let frame = thread.current_frame_mut().unwrap();
    frame
        .operand_stack
        .push(Slot::ReturnAddress((pc + 3) as u32));
    thread.pc = (pc as i32 + offset as i32) as usize;
    Ok(())
}

/// `jsr_w` (wide variant) pushes the address of the next instruction onto the
/// stack and jumps to another instruction.
///
/// The address of the next instruction is pushed onto the stack as a return address, 32-bit value.
pub fn jsr_w(thread: &mut Thread, offset: i32) -> Result<(), InstructionError> {
    let pc = thread.pc as u32;
    let frame = thread.current_frame_mut().unwrap();
    frame
        .operand_stack
        .push(Slot::ReturnAddress((pc + 5) as u32));
    thread.pc = (pc as i32 + offset) as usize;
    Ok(())
}

/// `ret` returns from a subroutine.
///
/// The index is an unsigned byte that must be an index into the local variable array of the current frame.
pub fn ret(thread: &mut Thread, index: u8) -> Result<(), InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let Slot::ReturnAddress(address) = frame.local_variables[index as usize] else {
        return Err(InstructionError::InvalidState {
            context: format!("Expected return address at index {}", index),
        });
    };
    thread.pc = address as usize;
    Ok(())
}

/// `tableswitch` accesses jump table by index and jumps.
pub fn tableswitch(thread: &mut Thread, table: &TableSwitch) -> Result<(), InstructionError> {
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
    thread.pc = (thread.pc as i32 + offset as i32) as usize;
    Ok(())
}

/// `lookupswitch` accesses jump table by key match and jumps.
pub fn lookupswitch(thread: &mut Thread, table: &LookupSwitch) -> Result<(), InstructionError> {
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
    thread.pc = (thread.pc as i32 + offset as i32) as usize;
    Ok(())
}

/// `return` returns void from a method.
pub fn vreturn(thread: &mut Thread) -> Result<(), InstructionError> {
    thread.pop_frame();
    // TODO: implement monitor strategy for synchronized methods
    let frame = thread.current_frame_mut().unwrap();
    let Some(Slot::InvokationReturnAddress(pc)) = frame.operand_stack.pop() else {
        return Err(InstructionError::InvalidState {
            context: "Expected invokation return address on the operand stack".into(),
        });
    };
    thread.pc = pc as usize;
    Ok(())
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
            pub fn $name(thread: &mut Thread) -> Result<(), InstructionError> {
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
                    thread.pc = pc as usize;
                    Ok(())
                } else {
                    return Err(InstructionError::InvalidState {
                        context: format!("Expected {:?} on the operand stack", stringify!($ty)),
                    });
                }
            }
        };
    }
}
