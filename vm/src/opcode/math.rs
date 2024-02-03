use super::{InstructionError, InstructionSuccess};
use crate::thread::Slot;
use crate::thread::Thread;
use crate::{xadd, xand, xdiv, xmul, xneg1, xneg2, xor, xrem, xshl, xshr, xsub, xxor};

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

xneg1!(ineg, Int);
xneg1!(lneg, Long);
xneg2!(fneg, Float, f32);
xneg2!(dneg, Double, f64);

xshl!(ishl, Int);
xshl!(lshl, Long);

xshr!(ishr, Int);
xshr!(lshr, Long);

// TODO: implement ushr

xand!(iand, Int);
xand!(land, Long);

xor!(ior, Int);
xor!(lor, Long);

xxor!(ixor, Int);
xxor!(lxor, Long);

/// `iinc` - Increment local variable by constant.
pub fn iinc(
    thread: &mut Thread,
    index: u8,
    increment: i8,
) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    if let Some(slot) = frame.local_variables.get_mut(index as usize) {
        if let Slot::Int(value) = slot {
            *value += increment as i32;
            Ok(InstructionSuccess::Next(3))
        } else {
            return Err(InstructionError::InvalidState {
                context: "Expected Int".into(),
            });
        }
    } else {
        return Err(InstructionError::InvalidState {
            context: "Local variable not found".into(),
        });
    }
}

/// `iinc` (wide variation) - Increment local variable by constant.
pub fn wide_iinc(
    thread: &mut Thread,
    index: u16,
    increment: i16,
) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    if let Some(slot) = frame.local_variables.get_mut(index as usize) {
        if let Slot::Int(value) = slot {
            *value += increment as i32;
            Ok(InstructionSuccess::Next(5))
        } else {
            return Err(InstructionError::InvalidState {
                context: "Expected Int".into(),
            });
        }
    } else {
        return Err(InstructionError::InvalidState {
            context: "Local variable not found".into(),
        });
    }
}

mod macros {
    #[macro_export]
    macro_rules! xadd {
        ($name:ident, $ty:ident, $real_ty:ty, $final_ty:ty) => {
            /// Add two values from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot1) = frame.operand_stack.pop() {
                    if let Some(slot2) = frame.operand_stack.pop() {
                        if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                            frame.operand_stack.push(Slot::$ty(
                                ((value1 as $real_ty) + (value2 as $real_ty)) as $final_ty,
                            ));
                        } else {
                            return Err(InstructionError::InvalidState {
                                context: format!("Expected {:?}", stringify!($ty)),
                            });
                        }
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: "Operand stack is len 1, expected as least two elements."
                                .into(),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: "Operand stack is empty".into(),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xsub {
        ($name:ident, $ty:ident, $real_ty:ty, $final_ty:ty) => {
            /// Substract two values from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot1) = frame.operand_stack.pop() {
                    if let Some(slot2) = frame.operand_stack.pop() {
                        if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                            frame.operand_stack.push(Slot::$ty(
                                ((value2 as $real_ty) - (value1 as $real_ty)) as $final_ty,
                            ));
                        } else {
                            return Err(InstructionError::InvalidState {
                                context: format!("Expected {:?}", stringify!($ty)),
                            });
                        }
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: "Operand stack is len 1, expected as least two elements."
                                .into(),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: "Operand stack is empty".into(),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xmul {
        ($name:ident, $ty:ident, $real_ty:ty, $final_ty:ty) => {
            /// Multiply two values from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot1) = frame.operand_stack.pop() {
                    if let Some(slot2) = frame.operand_stack.pop() {
                        if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                            frame.operand_stack.push(Slot::$ty(
                                ((value1 as $real_ty) * (value2 as $real_ty)) as $final_ty,
                            ));
                        } else {
                            return Err(InstructionError::InvalidState {
                                context: format!("Expected {:?}", stringify!($ty)),
                            });
                        }
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: "Operand stack is len 1, expected as least two elements."
                                .into(),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: "Operand stack is empty".into(),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xdiv {
        ($name:ident, $ty:ident, $real_ty:ty, $final_ty:ty) => {
            /// Divide a value by another from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot1) = frame.operand_stack.pop() {
                    if let Some(slot2) = frame.operand_stack.pop() {
                        if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                            frame.operand_stack.push(Slot::$ty(
                                ((value1 as $real_ty) / (value2 as $real_ty)) as $final_ty,
                            ));
                        } else {
                            return Err(InstructionError::InvalidState {
                                context: format!("Expected {:?}", stringify!($ty)),
                            });
                        }
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: "Operand stack is len 1, expected as least two elements."
                                .into(),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: "Operand stack is empty".into(),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xrem {
        ($name:ident, $ty:ident, $real_ty:ty, $final_ty:ty) => {
            /// The reminder of a value by another from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot1) = frame.operand_stack.pop() {
                    if let Some(slot2) = frame.operand_stack.pop() {
                        if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                            frame.operand_stack.push(Slot::$ty(
                                ((value1 as $real_ty) % (value2 as $real_ty)) as $final_ty,
                            ));
                        } else {
                            return Err(InstructionError::InvalidState {
                                context: format!("Expected {:?}", stringify!($ty)),
                            });
                        }
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: "Operand stack is len 1, expected as least two elements."
                                .into(),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: "Operand stack is empty".into(),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xneg1 {
        ($name:ident, $ty:ident) => {
            /// Negate a value from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot) = frame.operand_stack.pop() {
                    if let Slot::$ty(value) = slot {
                        frame.operand_stack.push(Slot::$ty(-value));
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: format!("Expected {:?}", stringify!($ty)),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: "Operand stack is empty".into(),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xneg2 {
        ($name:ident, $ty:ident, $real_ty:ty) => {
            /// Negate a value from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot) = frame.operand_stack.pop() {
                    if let Slot::$ty(value) = slot {
                        match value {
                            0.0 => frame.operand_stack.push(Slot::$ty(-0.0)),
                            -0.0 => frame.operand_stack.push(Slot::$ty(0.0)),
                            <$real_ty>::INFINITY => frame
                                .operand_stack
                                .push(Slot::$ty(<$real_ty>::NEG_INFINITY)),
                            <$real_ty>::NEG_INFINITY => {
                                frame.operand_stack.push(Slot::$ty(<$real_ty>::INFINITY))
                            }
                            x => frame.operand_stack.push(Slot::$ty(-x)),
                        }
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: format!("Expected {:?}", stringify!($ty)),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: "Operand stack is empty".into(),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xshl {
        ($name:ident, $ty:ident) => {
            /// Shift left a value from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot1) = frame.operand_stack.pop() {
                    if let Some(slot2) = frame.operand_stack.pop() {
                        if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                            frame
                                .operand_stack
                                .push(Slot::$ty(value1 << (value2 & 0x1f)));
                        } else {
                            return Err(InstructionError::InvalidState {
                                context: format!("Expected {:?}", stringify!($ty)),
                            });
                        }
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: "Operand stack is len 1, expected as least two elements."
                                .into(),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: "Operand stack is empty".into(),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xshr {
        ($name:ident, $ty:ident) => {
            /// Shift right a value from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot1) = frame.operand_stack.pop() {
                    if let Some(slot2) = frame.operand_stack.pop() {
                        if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                            frame
                                .operand_stack
                                .push(Slot::$ty(value1 >> (value2 & 0x1f)));
                        } else {
                            return Err(InstructionError::InvalidState {
                                context: format!("Expected {:?}", stringify!($ty)),
                            });
                        }
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: "Operand stack is len 1, expected as least two elements."
                                .into(),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: "Operand stack is empty".into(),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xand {
        ($name:ident, $ty:ident) => {
            /// Bitwise and a value from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let (Some(slot1), Some(slot2)) =
                    (frame.operand_stack.pop(), frame.operand_stack.pop())
                {
                    if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                        frame.operand_stack.push(Slot::$ty(value1 & value2));
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: format!("Expected {:?}", stringify!($ty)),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: "Operand stack is empty or len 1, expected as least two elements."
                            .into(),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xor {
        ($name:ident, $ty:ident) => {
            /// Bitwise or a value from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let (Some(slot1), Some(slot2)) =
                    (frame.operand_stack.pop(), frame.operand_stack.pop())
                {
                    if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                        frame.operand_stack.push(Slot::$ty(value1 | value2));
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: format!("Expected {:?}", stringify!($ty)),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: "Operand stack is empty or len 1, expected as least two elements."
                            .into(),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xxor {
        ($name:ident, $ty:ident) => {
            /// Bitwise xor a value from the operand stack and push the result onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let (Some(slot1), Some(slot2)) =
                    (frame.operand_stack.pop(), frame.operand_stack.pop())
                {
                    if let (Slot::$ty(value1), Slot::$ty(value2)) = (slot1, slot2) {
                        frame.operand_stack.push(Slot::$ty(value1 ^ value2));
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: format!("Expected {:?}", stringify!($ty)),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: "Operand stack is empty or len 1, expected as least two elements."
                            .into(),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }
}
