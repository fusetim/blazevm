use super::{InstructionError, InstructionSuccess};
use crate::thread::Slot;
use crate::thread::Thread;
use crate::{xstore, xstore_n};

xstore!(istore, Int);
xstore!(lstore, Long);
xstore!(fstore, Float);
xstore!(dstore, Double);
// TODO: implement astore

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

// TODO: implement astore_n
// TODO: implement array store instructions

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
}
