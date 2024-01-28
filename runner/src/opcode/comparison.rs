use super::InstructionError;
use crate::thread::Slot;
use crate::thread::Thread;
use crate::{if_icmpx, ifx};
use std::{f32, f64};

ifx!(ifeq, ==);
ifx!(ifne, !=);
ifx!(ifle, <=);
ifx!(iflt, <);
ifx!(ifge, >=);
ifx!(ifgt, >);

if_icmpx!(if_icmpeq, ==);
if_icmpx!(if_icmpne, !=);
if_icmpx!(if_icmple, <=);
if_icmpx!(if_icmplt, <);
if_icmpx!(if_icmpge, >=);
if_icmpx!(if_icmpgt, >);

// TODO: implement if_acmpx

/// `lcmp` compares two longs and pushes the result onto the stack.
pub fn lcmp(thread: &mut Thread) -> Result<(), InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let value2 = frame.operand_stack.pop().unwrap();
    let value1 = frame.operand_stack.pop().unwrap();
    let result = match (value1, value2) {
        (Slot::Long(value1), Slot::Long(value2)) => {
            if value1 > value2 {
                1
            } else if value1 == value2 {
                0
            } else {
                -1
            }
        }
        _ => {
            return Err(InstructionError::InvalidState {
                context: format!("Expected two longs."),
            })
        }
    };
    frame.operand_stack.push(Slot::Int(result));
    thread.pc += 1;
    Ok(())
}

/// `fcmpl` compares two floats and pushes the result onto the stack.
///
/// If either value is NaN, then -1 is pushed onto the stack.
pub fn fcmpl(thread: &mut Thread) -> Result<(), InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let value2 = frame.operand_stack.pop().unwrap();
    let value1 = frame.operand_stack.pop().unwrap();
    let result = match (value1, value2) {
        (Slot::Float(value1), Slot::Float(value2)) => {
            if value1 == f32::NAN || value2 == f32::NAN {
                -1
            } else if value1 > value2 {
                1
            } else if value1 == value2 {
                0
            } else {
                -1
            }
        }
        _ => {
            return Err(InstructionError::InvalidState {
                context: format!("Expected two floats."),
            })
        }
    };
    frame.operand_stack.push(Slot::Int(result));
    thread.pc += 1;
    Ok(())
}

/// `fcmpg` compares two floats and pushes the result onto the stack.
///
/// If either value is NaN, then 1 is pushed onto the stack.
pub fn fcmpg(thread: &mut Thread) -> Result<(), InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let value2 = frame.operand_stack.pop().unwrap();
    let value1 = frame.operand_stack.pop().unwrap();
    let result = match (value1, value2) {
        (Slot::Float(value1), Slot::Float(value2)) => {
            if value1 == f32::NAN || value2 == f32::NAN {
                1
            } else if value1 > value2 {
                1
            } else if value1 == value2 {
                0
            } else {
                -1
            }
        }
        _ => {
            return Err(InstructionError::InvalidState {
                context: format!("Expected two floats."),
            })
        }
    };
    frame.operand_stack.push(Slot::Int(result));
    thread.pc += 1;
    Ok(())
}

/// `dcmpl` compares two doubles and pushes the result onto the stack.
///
/// If either value is NaN, then -1 is pushed onto the stack.
pub fn dcmpl(thread: &mut Thread) -> Result<(), InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let value2 = frame.operand_stack.pop().unwrap();
    let value1 = frame.operand_stack.pop().unwrap();
    let result = match (value1, value2) {
        (Slot::Double(value1), Slot::Double(value2)) => {
            if value1 == f64::NAN || value2 == f64::NAN {
                -1
            } else if value1 > value2 {
                1
            } else if value1 == value2 {
                0
            } else {
                -1
            }
        }
        _ => {
            return Err(InstructionError::InvalidState {
                context: format!("Expected two floats"),
            })
        }
    };
    frame.operand_stack.push(Slot::Int(result));
    thread.pc += 1;
    Ok(())
}

/// `dcmpg` compares two doubles and pushes the result onto the stack.
///
/// If either value is NaN, then 1 is pushed onto the stack.
pub fn dcmpg(thread: &mut Thread) -> Result<(), InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let value2 = frame.operand_stack.pop().unwrap();
    let value1 = frame.operand_stack.pop().unwrap();
    let result = match (value1, value2) {
        (Slot::Double(value1), Slot::Double(value2)) => {
            if value1 == f64::NAN || value2 == f64::NAN {
                1
            } else if value1 > value2 {
                1
            } else if value1 == value2 {
                0
            } else {
                -1
            }
        }
        _ => {
            return Err(InstructionError::InvalidState {
                context: format!("Expected two floats."),
            })
        }
    };
    frame.operand_stack.push(Slot::Int(result));
    thread.pc += 1;
    Ok(())
}

mod macros {
    #[macro_export]
    macro_rules! ifx {
        ($name:ident, $cond:tt) => {
            /// Branch if top of stack comparison with zero succeeds.
            pub fn $name(thread: &mut Thread, offset: i16) -> Result<(), InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(Slot::Int(value)) = frame.operand_stack.pop() {
                    if value $cond 0 {
                        thread.pc = (thread.pc as i32 + offset as i32) as usize;
                    } else {
                        thread.pc += 3;
                    }
                    Ok(())
                } else {
                    Err(InstructionError::InvalidState { context: "Expected int on top of operand stack".into() })
                }
            }
        };
    }

    #[macro_export]
    macro_rules! if_icmpx {
        ($name:ident, $cond:tt) => {
            /// Branch if int comparison succeeds.
            pub fn $name(thread: &mut Thread, offset: i16) -> Result<(), InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(Slot::Int(value2)) = frame.operand_stack.pop() {
                    if let Some(Slot::Int(value1)) = frame.operand_stack.pop() {
                        if value1 $cond value2 {
                            thread.pc = (thread.pc as i32 + offset as i32) as usize;
                        } else {
                            thread.pc += 3;
                        }
                        Ok(())
                    } else {
                        Err(InstructionError::InvalidState { context: "Expected int on top of operand stack".into() })
                    }
                } else {
                    Err(InstructionError::InvalidState { context: "Expected int on top of operand stack".into() })
                }
            }
        };
    }
}
