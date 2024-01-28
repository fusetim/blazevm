use super::InstructionError;
use crate::class_manager::ClassManager;
use crate::thread::Slot;
use crate::thread::Thread;
use crate::{xload, xload_n};

xload!(iload, Int);
xload!(lload, Long);
xload!(fload, Float);
xload!(dload, Double);
// TODO: implement aload

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

// TODO: implement aload_n
// TODO: implement array load instructions

mod macros {
    #[macro_export]
    macro_rules! xload {
        ($name:ident, $ty:ident) => {
            /// Load a value from the local variables onto the operand stack.
            pub fn $name(thread: &mut Thread, index: u8) -> Result<(), InstructionError> {
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
                thread.pc += 2;
                Ok(())
            }
        };
    }

    #[macro_export]
    macro_rules! xload_n {
        ($name:ident, $ty:ident, $index:expr) => {
            /// Load a value from the local variables onto the operand stack.
            pub fn $name(thread: &mut Thread) -> Result<(), InstructionError> {
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
                thread.pc += 2;
                Ok(())
            }
        };
    }
}
