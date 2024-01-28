use crate::thread::Thread;
use crate::thread::Slot;
use super::InstructionError;
use crate::{x2y, i2truncate};

x2y!(i2l, Int, Long, i64);
x2y!(i2f, Int, Float, f32);
x2y!(i2d, Int, Double, f64);

x2y!(l2i, Long, Int, i32);
x2y!(l2f, Long, Float, f32);
x2y!(l2d, Long, Double, f64);

x2y!(f2i, Float, Int, i32);
x2y!(f2l, Float, Long, i64);
x2y!(f2d, Float, Double, f64);

x2y!(d2i, Double, Int, i32);
x2y!(d2l, Double, Long, i64);
x2y!(d2f, Double, Float, f32);

i2truncate!(i2b, i8);
i2truncate!(i2c, u16);
i2truncate!(i2s, i16);

mod macros {
    #[macro_export]
    macro_rules! x2y {
        ($name:ident, $srcty:ident, $destty:ident, $real_destty:ty) => {
            /// Convert the top value to another numeric form and push it back to the stack.
            pub fn $name(thread: &mut Thread) -> Result<(), InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot) = frame.operand_stack.pop() {
                    if let Slot::$srcty(value) = slot {
                        frame.operand_stack.push(Slot::$destty(value as $real_destty));
                        thread.pc += 1;
                        Ok(())
                    } else {
                        return Err(InstructionError::InvalidState { context: format!("Expected {:?} but got {:?}", stringify!($srcty), slot) });
                    }
                } else {
                    return Err(InstructionError::InvalidState { context: "Operand stack is empty".into() });
                }
            }
        };
    }

    #[macro_export]
    macro_rules! i2truncate {
        ($name:ident, $real_destty:ty) => {
            /// Convert the top value (int) to a byte/char/short form by truncation and push it back to the stack.
            pub fn $name(thread: &mut Thread) -> Result<(), InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                if let Some(slot) = frame.operand_stack.pop() {
                    if let Slot::Int(value) = slot {
                        frame.operand_stack.push(Slot::Int((value as $real_destty) as i32));
                        thread.pc += 1;
                        Ok(())
                    } else {
                        return Err(InstructionError::InvalidState { context: format!("Expected {:?} but got {:?}", stringify!($ty), slot) });
                    }
                } else {
                    return Err(InstructionError::InvalidState { context: "Operand stack is empty".into() });
                }
            }
        };
    }
}