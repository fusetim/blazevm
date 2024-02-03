use super::{InstructionError, InstructionSuccess};
use crate::alloc::Array;
use crate::thread::Slot;
use crate::thread::Thread;
use crate::{astore_n, xastore, xstore, xstore_n};

xstore!(istore, Int);
xstore!(lstore, Long);
xstore!(fstore, Float);
xstore!(dstore, Double);

xstore_n!(istore_0, Int, 0);
xstore_n!(istore_1, Int, 1);
xstore_n!(istore_2, Int, 2);
xstore_n!(istore_3, Int, 3);

xstore_n!(lstore_0, Long, 0, true);
xstore_n!(lstore_1, Long, 1, true);
xstore_n!(lstore_2, Long, 2, true);
xstore_n!(lstore_3, Long, 3, true);

xstore_n!(fstore_0, Float, 0);
xstore_n!(fstore_1, Float, 1);
xstore_n!(fstore_2, Float, 2);
xstore_n!(fstore_3, Float, 3);

xstore_n!(dstore_0, Double, 0, true);
xstore_n!(dstore_1, Double, 1, true);
xstore_n!(dstore_2, Double, 2, true);
xstore_n!(dstore_3, Double, 3, true);

astore_n!(astore_0, 0);
astore_n!(astore_1, 1);
astore_n!(astore_2, 2);
astore_n!(astore_3, 3);

xastore!(iastore, Int, Int, i32);
xastore!(lastore, Long, Long, i64);
xastore!(fastore, Float, Float, f32);
xastore!(dastore, Double, Double, f64);
xastore!(castore, Int, Char, u16);
xastore!(sastore, Int, Short, i16);

// TODO: implement array store instructions

/// Store a reference from the operand stack into the local variables.
pub fn astore(thread: &mut Thread, index: u8) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    if let Some(slot) = frame.operand_stack.pop() {
        if slot.is_reference() {
            if frame.local_variables.len() <= index as usize {
                return Err(InstructionError::InvalidState { context: format!("Index out of bound, the local variable array is len: {}, index given is: {}.", frame.local_variables.len(), index) });
            }
            frame.local_variables[index as usize] = slot;
        } else {
            return Err(InstructionError::InvalidState {
                context: format!("Expected reference but got {:?}", slot),
            });
        }
    } else {
        return Err(InstructionError::InvalidState {
            context: "Operand stack is empty".into(),
        });
    }
    Ok(InstructionSuccess::Next(2))
}

/// Store a reference from the operand stack into an array.
pub fn aastore(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let value = frame
        .operand_stack
        .pop()
        .ok_or_else(|| InstructionError::InvalidState {
            context: "Expected value on the operand stack".into(),
        })?;
    let Some(Slot::Int(index)) = frame.operand_stack.pop() else {
        return Err(InstructionError::InvalidState {
            context: "Expected index on the operand stack".into(),
        });
    };
    let Some(Slot::ArrayReference(array_ref)) = frame.operand_stack.pop() else {
        return Err(InstructionError::InvalidState {
            context: "Expected arrayref on the operand stack".into(),
        });
    };
    match array_ref.as_ref() {
        &Array::ArrayRef(ref array) => match value {
            Slot::ArrayReference(value) => {
                array.set(index as usize, Some(value));
            }
            Slot::UndefinedReference => {
                array.set(index as usize, None);
            }
            _ => {
                return Err(InstructionError::InvalidState {
                    context: format!("Expected reference but got {:?}", value),
                });
            }
        },
        &Array::ObjectRef(ref array) => {
            // TODO: Check if the actual type of the object is compatible with the array type.
            match value {
                Slot::ObjectReference(value) => {
                    array.set(index as usize, Some(value));
                }
                Slot::UndefinedReference => {
                    array.set(index as usize, None);
                }
                _ => {
                    return Err(InstructionError::InvalidState {
                        context: format!("Expected reference but got {:?}", value),
                    });
                }
            }
        }
        _ => {
            return Err(InstructionError::InvalidState {
                context: format!("Expected reference array but got {:?}", array_ref),
            });
        }
    }
    Ok(InstructionSuccess::Next(1))
}

/// Store a bool/byte from the operand stack into an array.
pub fn bastore(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let value = frame
        .operand_stack
        .pop()
        .ok_or_else(|| InstructionError::InvalidState {
            context: "Expected value on the operand stack".into(),
        })?;
    let Some(Slot::Int(index)) = frame.operand_stack.pop() else {
        return Err(InstructionError::InvalidState {
            context: "Expected index on the operand stack".into(),
        });
    };
    let Some(Slot::ArrayReference(array_ref)) = frame.operand_stack.pop() else {
        return Err(InstructionError::InvalidState {
            context: "Expected arrayref on the operand stack".into(),
        });
    };
    match array_ref.as_ref() {
        &Array::Byte(ref array) => match value {
            Slot::Int(value) => {
                array.set(index as usize, value as i8);
            }
            _ => {
                return Err(InstructionError::InvalidState {
                    context: format!("Expected byte but got {:?}", value),
                });
            }
        },
        &Array::Boolean(ref array) => match value {
            Slot::Int(value) => {
                array.set(index as usize, value != 0);
            }
            _ => {
                return Err(InstructionError::InvalidState {
                    context: format!("Expected boolean but got {:?}", value),
                });
            }
        },
        _ => {
            return Err(InstructionError::InvalidState {
                context: format!("Expected byte array but got {:?}", array_ref),
            });
        }
    }
    Ok(InstructionSuccess::Next(1))
}

mod macros {
    #[macro_export]
    macro_rules! xstore {
        ($name:ident, $ty:ident) => {
            /// Store a value from the operand stack into the local variables.
            pub fn $name(thread: &mut Thread, index: u8) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot) = frame.operand_stack.pop() {
                    if let Slot::$ty(value) = slot {
                        if frame.local_variables.len() <= index as usize {
                            return Err(InstructionError::InvalidState { context: format!("Index out of bound, the local variable array is len: {}, index given is: {}.", frame.local_variables.len(), index) });
                        }
                        frame.local_variables[index as usize] = Slot::$ty(value);
                    } else {
                        return Err(InstructionError::InvalidState { context: format!("Expected {:?} but got {:?}", stringify!($ty), slot) });
                    }
                } else {
                    return Err(InstructionError::InvalidState { context: "Operand stack is empty".into() });
                }
                Ok(InstructionSuccess::Next(2))
            }
        };

        ($name:ident, $ty:ident, true) => {
            /// Store a value from the operand stack into the local variables.
            pub fn $name(thread: &mut Thread, index: u8) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot) = frame.operand_stack.pop() {
                    if let Slot::$ty(value) = slot {
                        if frame.local_variables.len() <= (index + 1) as usize {
                            return Err(InstructionError::InvalidState { context: format!("Index out of bound, the local variable array is len: {}, index given is: {}.", frame.local_variables.len(), index) });
                        }
                        frame.local_variables[index as usize] = Slot::$ty(value);
                        frame.local_variables[index as usize + 1] = Slot::Tombstone;
                    } else {
                        return Err(InstructionError::InvalidState { context: format!("Expected {:?} but got {:?}", stringify!($ty), slot) });
                    }
                } else {
                    return Err(InstructionError::InvalidState { context: "Operand stack is empty".into() });
                }
                Ok(InstructionSuccess::Next(2))
            }
        };
    }

    #[macro_export]
    macro_rules! xstore_n {
        ($name:ident, $ty:ident, $index:expr) => {
            /// Store a value from the operand stack into the local variables.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot) = frame.operand_stack.pop() {
                    if let Slot::$ty(value) = slot {
                        if frame.local_variables.len() <= $index as usize {
                            return Err(InstructionError::InvalidState { context: format!("Index out of bound, the local variable array is len: {}, index given is: {}.", frame.local_variables.len(), $index) });
                        }
                        frame.local_variables[$index as usize] = Slot::$ty(value);
                    } else {
                        return Err(InstructionError::InvalidState { context: format!("Expected {:?} but got {:?}", stringify!($ty), slot) });
                    }
                } else {
                    return Err(InstructionError::InvalidState { context: "Operand stack is empty".into() });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };

        ($name:ident, $ty:ident, $index:expr, true) => {
            /// Store a value from the operand stack into the local variables.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot) = frame.operand_stack.pop() {
                    if let Slot::$ty(value) = slot {
                        if frame.local_variables.len() <= ($index + 1) as usize {
                            return Err(InstructionError::InvalidState { context: format!("Index out of bound, the local variable array is len: {}, index given is: {}.", frame.local_variables.len(), $index) });
                        }
                        frame.local_variables[$index as usize] = Slot::$ty(value);
                        frame.local_variables[$index as usize + 1] = Slot::Tombstone;
                    } else {
                        return Err(InstructionError::InvalidState { context: format!("Expected {:?} but got {:?}", stringify!($ty), slot) });
                    }
                } else {
                    return Err(InstructionError::InvalidState { context: "Operand stack is empty".into() });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! astore_n {
        ($name:ident, $index:expr) => {
            /// Store a value from the operand stack into the local variables.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot) = frame.operand_stack.pop() {
                    if slot.is_reference() {
                        if frame.local_variables.len() <= $index as usize {
                            return Err(InstructionError::InvalidState { context: format!("Index out of bound, the local variable array is len: {}, index given is: {}.", frame.local_variables.len(), $index) });
                        }
                        frame.local_variables[$index as usize] = slot;
                    } else {
                        return Err(InstructionError::InvalidState { context: format!("Expected reference but got {:?}", slot) });
                    }
                } else {
                    return Err(InstructionError::InvalidState { context: "Operand stack is empty".into() });
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }

    #[macro_export]
    macro_rules! xastore {
        ($name:ident, $ty:ident, $arrty:ident, $convty:ty) => {
            /// Store a value from the operand stack into the local variables.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                let value =
                    frame
                        .operand_stack
                        .pop()
                        .ok_or_else(|| InstructionError::InvalidState {
                            context: "Expected value on the operand stack".into(),
                        })?;
                let Some(Slot::Int(index)) = frame.operand_stack.pop() else {
                    return Err(InstructionError::InvalidState {
                        context: "Expected index on the operand stack".into(),
                    });
                };
                let Some(Slot::ArrayReference(array_ref)) = frame.operand_stack.pop() else {
                    return Err(InstructionError::InvalidState {
                        context: "Expected arrayref on the operand stack".into(),
                    });
                };
                match array_ref.as_ref() {
                    &Array::$arrty(ref array) => {
                        if let Slot::$ty(value) = value {
                            array.set(index as usize, value as $convty);
                        } else {
                            return Err(InstructionError::InvalidState {
                                context: format!(
                                    "Expected {:?} but got {:?}",
                                    stringify!($ty),
                                    value
                                ),
                            });
                        }
                    }
                    _ => {
                        return Err(InstructionError::InvalidState {
                            context: format!(
                                "Expected {:?} but got {:?}",
                                stringify!($arrty),
                                array_ref
                            ),
                        });
                    }
                }
                Ok(InstructionSuccess::Next(1))
            }
        };
    }
}
