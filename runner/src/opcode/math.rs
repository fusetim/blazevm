use crate::thread::Thread;
use crate::thread::Slot;
use super::InstructionError;
use crate::{xadd, xsub, xmul, xdiv, xrem};

xadd!(iadd, Int, i32, i32);
xadd!(ladd, Long, i64, i64);
xadd!(fadd, Float, f32, f32);
xadd!(dadd, Double, f64, f64);

xsub!(isub, Int, i32, i32);
xsub!(lsub, Long, i64, i64);
xsub!(fsub, Float, f32, f32);
xsub!(dsub, Double, f64, f64);

xmul!(imul, Int, i32, i32);
xmul!(lmul, Long, i64, i64);
xmul!(fmul, Float, f32, f32);
xmul!(dmul, Double, f64, f64);

xdiv!(idiv, Int, i32, i32);
xdiv!(ldiv, Long, i64, i64);
xdiv!(fdiv, Float, f32, f32);
xdiv!(ddiv, Double, f64, f64);

xrem!(irem, Int, i32, i32);
xrem!(lrem, Long, i64, i64);
xrem!(frem, Float, f32, f32);
xrem!(drem, Double, f64, f64);

// TODO: implement shl, shr, ushl, ushr, and, or, xor, iinc

mod macros {
    #[macro_export]
    macro_rules! xadd {
        ($name:ident, $ty:ident, $real_ty:ty, $final_ty:ty) => {
            /// Add two values from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<(), InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot1) = frame.operand_stack.pop() {
                    if let Some(slot2) = frame.operand_stack.pop() {
                        if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                            frame.operand_stack.push(Slot::$ty(((value1 as $real_ty) + (value2 as $real_ty)) as $final_ty));
                        } else {
                            return Err(InstructionError::InvalidState { context: format!("Expected {:?}", stringify!($ty)) });
                        }
                    } else {
                        return Err(InstructionError::InvalidState { context: "Operand stack is len 1, expected as least two elements.".into() });
                    }
                } else {
                    return Err(InstructionError::InvalidState { context: "Operand stack is empty".into() });
                }
                thread.pc += 1;
                Ok(())
            }
        };
    }

    #[macro_export]
    macro_rules! xsub {
        ($name:ident, $ty:ident, $real_ty:ty, $final_ty:ty) => {
            /// Substract two values from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<(), InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot1) = frame.operand_stack.pop() {
                    if let Some(slot2) = frame.operand_stack.pop() {
                        if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                            frame.operand_stack.push(Slot::$ty(((value1 as $real_ty) - (value2 as $real_ty)) as $final_ty));
                        } else {
                            return Err(InstructionError::InvalidState { context: format!("Expected {:?}", stringify!($ty)) });
                        }
                    } else {
                        return Err(InstructionError::InvalidState { context: "Operand stack is len 1, expected as least two elements.".into() });
                    }
                } else {
                    return Err(InstructionError::InvalidState { context: "Operand stack is empty".into() });
                }
                thread.pc += 1;
                Ok(())
            }
        };
    }

    #[macro_export]
    macro_rules! xmul {
        ($name:ident, $ty:ident, $real_ty:ty, $final_ty:ty) => {
            /// Multiply two values from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<(), InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot1) = frame.operand_stack.pop() {
                    if let Some(slot2) = frame.operand_stack.pop() {
                        if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                            frame.operand_stack.push(Slot::$ty(((value1 as $real_ty) * (value2 as $real_ty)) as $final_ty));
                        } else {
                            return Err(InstructionError::InvalidState { context: format!("Expected {:?}", stringify!($ty)) });
                        }
                    } else {
                        return Err(InstructionError::InvalidState { context: "Operand stack is len 1, expected as least two elements.".into() });
                    }
                } else {
                    return Err(InstructionError::InvalidState { context: "Operand stack is empty".into() });
                }
                thread.pc += 1;
                Ok(())
            }
        };
    }

    #[macro_export]
    macro_rules! xdiv {
        ($name:ident, $ty:ident, $real_ty:ty, $final_ty:ty) => {
            /// Divide a value by another from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<(), InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot1) = frame.operand_stack.pop() {
                    if let Some(slot2) = frame.operand_stack.pop() {
                        if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                            frame.operand_stack.push(Slot::$ty(((value1 as $real_ty) / (value2 as $real_ty)) as $final_ty));
                        } else {
                            return Err(InstructionError::InvalidState { context: format!("Expected {:?}", stringify!($ty)) });
                        }
                    } else {
                        return Err(InstructionError::InvalidState { context: "Operand stack is len 1, expected as least two elements.".into() });
                    }
                } else {
                    return Err(InstructionError::InvalidState { context: "Operand stack is empty".into() });
                }
                thread.pc += 1;
                Ok(())
            }
        };
    }

    #[macro_export]
    macro_rules! xrem {
        ($name:ident, $ty:ident, $real_ty:ty, $final_ty:ty) => {
            /// The reminder of a value by another from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<(), InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot1) = frame.operand_stack.pop() {
                    if let Some(slot2) = frame.operand_stack.pop() {
                        if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                            frame.operand_stack.push(Slot::$ty(((value1 as $real_ty) % (value2 as $real_ty)) as $final_ty));
                        } else {
                            return Err(InstructionError::InvalidState { context: format!("Expected {:?}", stringify!($ty)) });
                        }
                    } else {
                        return Err(InstructionError::InvalidState { context: "Operand stack is len 1, expected as least two elements.".into() });
                    }
                } else {
                    return Err(InstructionError::InvalidState { context: "Operand stack is empty".into() });
                }
                thread.pc += 1;
                Ok(())
            }
        };
    }

}