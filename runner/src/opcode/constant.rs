
use crate::xconst_i;
use crate::thread::Thread;
use crate::class_manager::ClassManager;
use crate::thread::Slot;
use super::InstructionError;

pub fn nop(thread: &mut Thread, _cm: &mut ClassManager) -> Result<(), InstructionError> {
    thread.pc += 1;
    Ok(())
}

xconst_i!(iconst_m1, Int, -1);
xconst_i!(iconst_0,  Int, 0);
xconst_i!(iconst_1,  Int, 1);
xconst_i!(iconst_2,  Int, 2);
xconst_i!(iconst_3,  Int, 3);
xconst_i!(iconst_4,  Int, 4);
xconst_i!(iconst_5,  Int, 5);

xconst_i!(lconst_0, Long, 0);
xconst_i!(lconst_1, Long, 1);

xconst_i!(fconst_0, Float, 0.0);
xconst_i!(fconst_1, Float, 1.0);
xconst_i!(fconst_2, Float, 2.0);

xconst_i!(dconst_0, Double, 0.0);
xconst_i!(dconst_1, Double, 1.0);

pub fn bipush(thread: &mut Thread, _cm: &mut ClassManager, value: i8) -> Result<(), InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    frame.operand_stack.push(Slot::Int(value as i32));
    thread.pc += 2;
    Ok(())
}

pub fn sipush(thread: &mut Thread, _cm: &mut ClassManager, value: i16) -> Result<(), InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    frame.operand_stack.push(Slot::Int(value as i32));
    thread.pc += 3;
    Ok(())
}

mod macros {
    #[macro_export]
    macro_rules! xconst_i {
        ($name:ident, $sloty:ident, $value:expr) => {
            pub fn $name(thread: &mut Thread, _cm: &mut ClassManager) -> Result<(), InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                frame.operand_stack.push(Slot::$sloty($value));
                thread.pc += 1;
                Ok(())
            }
        };
    }
}