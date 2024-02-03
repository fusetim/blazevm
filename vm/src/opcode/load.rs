use super::{InstructionError, InstructionSuccess};
use crate::alloc::Array;
use crate::thread::Slot;
use crate::thread::Thread;
use crate::{aload_n, xaload, xload, xload_n};

xload!(iload, Int);
xload!(lload, Long);
xload!(fload, Float);
xload!(dload, Double);

xload_n!(iload_0, Int, 0);
xload_n!(iload_1, Int, 1);
xload_n!(iload_2, Int, 2);
xload_n!(iload_3, Int, 3);

xload_n!(lload_0, Long, 0);
xload_n!(lload_1, Long, 1);
xload_n!(lload_2, Long, 2);
xload_n!(lload_3, Long, 3);

xload_n!(fload_0, Float, 0);
xload_n!(fload_1, Float, 1);
xload_n!(fload_2, Float, 2);
xload_n!(fload_3, Float, 3);

xload_n!(dload_0, Double, 0);
xload_n!(dload_1, Double, 1);
xload_n!(dload_2, Double, 2);
xload_n!(dload_3, Double, 3);

aload_n!(aload_0, 0);
aload_n!(aload_1, 1);
aload_n!(aload_2, 2);
aload_n!(aload_3, 3);

xaload!(iaload, Int, Int, i32);
xaload!(laload, Long, Long, i64);
xaload!(faload, Float, Float, f32);
xaload!(daload, Double, Double, f64);
xaload!(caload, Int, Char, i32);
xaload!(saload, Int, Short, i32);

/// Load a reference from the local variables onto the operand stack.
pub fn aload(thread: &mut Thread, index: u8) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    if let Some(slot) = frame.local_variables.get(index as usize) {
        if slot.is_reference() {
            frame.operand_stack.push(slot.clone());
        } else {
            return Err(InstructionError::InvalidState {
                context: format!("Expected reference but got {:?}", slot),
            });
        }
    } else {
        return Err(InstructionError::InvalidState {
            context: format!("Local variable {} not found", index),
        });
    }
    Ok(InstructionSuccess::Next(2))
}

/// Load a bool/byte from the local variables onto the operand stack.
pub fn baload(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let Some(Slot::Int(index)) = frame.operand_stack.pop() else {
        return Err(InstructionError::InvalidState {
            context: "Expected index on the operand stack".into(),
        });
    };
    let arrayref = frame
        .operand_stack
        .pop()
        .ok_or_else(|| InstructionError::InvalidState {
            context: "Expected arrayref on the operand stack".into(),
        })?;
    if let Slot::ArrayReference(ref array) = arrayref {
        match array.as_ref() {
            &Array::Byte(ref arr) => {
                let value =
                    arr.get(index as usize)
                        .ok_or_else(|| InstructionError::InvalidState {
                            context: "Index out of bounds".into(),
                        })?;
                frame.operand_stack.push(Slot::Int(value as i32));
            }
            &Array::Boolean(ref arr) => {
                let value =
                    arr.get(index as usize)
                        .ok_or_else(|| InstructionError::InvalidState {
                            context: "Index out of bounds".into(),
                        })?;
                if value {
                    frame.operand_stack.push(Slot::Int(1));
                } else {
                    frame.operand_stack.push(Slot::Int(0));
                }
            }
            _ => {
                return Err(InstructionError::InvalidState {
                    context: format!("Expected arrayref but got {:?}", arrayref),
                });
            }
        }
    } else {
        return Err(InstructionError::InvalidState {
            context: format!("Expected arrayref but got {:?}", arrayref),
        });
    }
    Ok(InstructionSuccess::Next(1))
}

/// Load a reference from an array.
pub fn aaload(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let Some(Slot::Int(index)) = frame.operand_stack.pop() else {
        return Err(InstructionError::InvalidState {
            context: "Expected index on the operand stack".into(),
        });
    };
    let arrayref = frame
        .operand_stack
        .pop()
        .ok_or_else(|| InstructionError::InvalidState {
            context: "Expected arrayref on the operand stack".into(),
        })?;
    if let Slot::ArrayReference(ref array) = arrayref {
        match array.as_ref() {
            Array::ObjectRef(objref) => {
                if let Some(obj) =
                    objref
                        .get(index as usize)
                        .ok_or_else(|| InstructionError::InvalidState {
                            context: "Index out of bounds".into(),
                        })?
                {
                    frame.operand_stack.push(Slot::ObjectReference(obj));
                } else {
                    frame.operand_stack.push(Slot::UndefinedReference);
                }
            }
            Array::ArrayRef(aref) => {
                if let Some(arr) =
                    aref.get(index as usize)
                        .ok_or_else(|| InstructionError::InvalidState {
                            context: "Index out of bounds".into(),
                        })?
                {
                    frame.operand_stack.push(Slot::ArrayReference(arr));
                } else {
                    frame.operand_stack.push(Slot::UndefinedReference);
                }
            }
            _ => {
                return Err(InstructionError::InvalidState {
                    context: format!("Expected arrayref but got {:?}", arrayref),
                });
            }
        }
    } else {
        return Err(InstructionError::InvalidState {
            context: format!("Expected arrayref but got {:?}", arrayref),
        });
    }
    Ok(InstructionSuccess::Next(1))
}

mod macros {
    #[macro_export]
    macro_rules! xload {
        ($name:ident, $ty:ident) => {
            /// Load a value from the local variables onto the operand stack.
            pub fn $name(
                thread: &mut Thread,
                index: u8,
            ) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot) = frame.local_variables.get(index as usize) {
                    if let Slot::$ty(value) = slot {
                        frame.operand_stack.push(Slot::$ty(*value));
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: format!("Expected {:?} but got {:?}", stringify!($ty), slot),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: format!("Local variable {} not found", index),
                    });
                }
                Ok(InstructionSuccess::Next(2))
            }
        };
    }

    #[macro_export]
    macro_rules! xload_n {
        ($name:ident, $ty:ident, $index:expr) => {
            /// Load a value from the local variables onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot) = frame.local_variables.get($index as usize) {
                    if let Slot::$ty(value) = slot {
                        frame.operand_stack.push(Slot::$ty(*value));
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: format!("Expected {:?} but got {:?}", stringify!($ty), slot),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: format!("Local variable {} not found", $index),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! aload_n {
        ($name:ident, $index:expr) => {
            /// Load a value from the local variables onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot) = frame.local_variables.get($index as usize) {
                    if slot.is_reference() {
                        frame.operand_stack.push(slot.clone());
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: format!("Expected reference but got {:?}", slot),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: format!("Local variable {} not found", $index),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xaload {
        ($name:ident, $ty:ident, $arrty:ident, $convty:ty) => {
            /// Load a value from an array onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                let Some(Slot::Int(index)) = frame.operand_stack.pop() else {
                    return Err(InstructionError::InvalidState {
                        context: "Expected index on the operand stack".into(),
                    });
                };
                let arrayref =
                    frame
                        .operand_stack
                        .pop()
                        .ok_or_else(|| InstructionError::InvalidState {
                            context: "Expected arrayref on the operand stack".into(),
                        })?;
                if let Slot::ArrayReference(ref array) = arrayref {
                    if let Array::$arrty(array) = array.as_ref() {
                        let value = array.get(index as usize).ok_or_else(|| {
                            InstructionError::InvalidState {
                                context: "Index out of bounds".into(),
                            }
                        })?;
                        frame.operand_stack.push(Slot::$ty(value as $convty));
                    } else {
                        return Err(InstructionError::InvalidState {
                            context: format!("Expected arrayref but got {:?}", arrayref),
                        });
                    }
                } else {
                    return Err(InstructionError::InvalidState {
                        context: format!("Expected arrayref but got {:?}", arrayref),
                    });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }
}
