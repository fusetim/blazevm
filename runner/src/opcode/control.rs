use super::InstructionError;
use crate::thread::Slot;
use crate::thread::Thread;

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
    frame.operand_stack.push(Slot::ReturnAddress((pc + 3) as u32));
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
    frame.operand_stack.push(Slot::ReturnAddress((pc + 5) as u32));
    thread.pc = (pc as i32 + offset) as usize;
    Ok(())
}

/// `ret` returns from a subroutine.
///
/// The index is an unsigned byte that must be an index into the local variable array of the current frame.
pub fn ret(thread: &mut Thread, index: u8) -> Result<(), InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let Slot::ReturnAddress(address) = frame.local_variables[index as usize]
    else {
        return Err(InstructionError::InvalidState {
            context: format!("Expected return address at index {}", index),
        });
    };
    thread.pc = address as usize;
    Ok(())
}

// TODO: implement tableswitch, lookupswitch

/// `return` returns void from a method.
pub fn vreturn(thread: &mut Thread) -> Result<(), InstructionError> {
    thread.pop_frame();
    /// TODO: implement monitor strategy for synchronized methods
    /// TODO: We might need to reset the pc to the return address of the previous frame?
    Ok(())
}