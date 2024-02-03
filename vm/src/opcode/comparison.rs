use super::{InstructionError, InstructionSuccess};
use crate::thread::Slot;
use crate::thread::Thread;
use crate::{if_acmpx, if_icmpx, ifx};
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

if_acmpx!(if_acmpeq, true);
if_acmpx!(if_acmpne, false);

/// `lcmp` compares two longs and pushes the result onto the stack.
pub fn lcmp(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
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
    Ok(InstructionSuccess::Next(1))
}

/// `fcmpl` compares two floats and pushes the result onto the stack.
///
/// If either value is NaN, then -1 is pushed onto the stack.
pub fn fcmpl(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
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
    Ok(InstructionSuccess::Next(1))
}

/// `fcmpg` compares two floats and pushes the result onto the stack.
///
/// If either value is NaN, then 1 is pushed onto the stack.
pub fn fcmpg(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
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
    Ok(InstructionSuccess::Next(1))
}

/// `dcmpl` compares two doubles and pushes the result onto the stack.
///
/// If either value is NaN, then -1 is pushed onto the stack.
pub fn dcmpl(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
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
    Ok(InstructionSuccess::Next(1))
}

/// `dcmpg` compares two doubles and pushes the result onto the stack.
///
/// If either value is NaN, then 1 is pushed onto the stack.
pub fn dcmpg(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
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
    Ok(InstructionSuccess::Next(1))
}

mod macros {
    #[macro_export]
    macro_rules! ifx {
        ($name:ident, $cond:tt) => {
            /// Branch if top of stack comparison with zero succeeds.
            pub fn $name(thread: &mut Thread, offset: i16) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(Slot::Int(value)) = frame.operand_stack.pop() {
                    if value $cond 0 {
                        Ok(InstructionSuccess::JumpRelative(offset as isize))
                    } else {
                        Ok(InstructionSuccess::Next(3))
                    }
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
            pub fn $name(thread: &mut Thread, offset: i16) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(Slot::Int(value2)) = frame.operand_stack.pop() {
                    if let Some(Slot::Int(value1)) = frame.operand_stack.pop() {
                        if value1 $cond value2 {
                            Ok(InstructionSuccess::JumpRelative(offset as isize))
                        } else {
                            Ok(InstructionSuccess::Next(3))
                        }
                    } else {
                        Err(InstructionError::InvalidState { context: "Expected int on top of operand stack".into() })
                    }
                } else {
                    Err(InstructionError::InvalidState { context: "Expected int on top of operand stack".into() })
                }
            }
        };
    }

    #[macro_export]
    macro_rules! if_acmpx {
        ($name:ident, $on_eq:tt) => {
            /// Branch if reference comparison succeeds.
            pub fn $name(
                thread: &mut Thread,
                offset: i16,
            ) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(value2) = frame.operand_stack.pop() {
                    if let Some(value1) = frame.operand_stack.pop() {
                        let eqcheck = match (value1, value2) {
                            (Slot::UndefinedReference, Slot::UndefinedReference) => true,
                            (Slot::ObjectReference(obj1), Slot::ObjectReference(obj2)) => {
                                std::ptr::eq(obj1.as_ref(), obj2.as_ref())
                            }
                            (Slot::ArrayReference(arr1), Slot::ArrayReference(arr2)) => {
                                std::ptr::eq(arr1.as_ref(), arr2.as_ref())
                            }
                            (x, y) if x.is_reference() && y.is_reference() => false,
                            _ => {
                                return Err(InstructionError::InvalidState {
                                    context: "Expected reference on top of operand stack".into(),
                                });
                            }
                        };
                        if eqcheck == $on_eq {
                            Ok(InstructionSuccess::JumpRelative(offset as isize))
                        } else {
                            Ok(InstructionSuccess::Next(3))
                        }
                    } else {
                        Err(InstructionError::InvalidState {
                            context: "Expected reference on top of operand stack".into(),
                        })
                    }
                } else {
                    Err(InstructionError::InvalidState {
                        context: "Expected reference on top of operand stack".into(),
                    })
                }
            }
        };
    }
}
