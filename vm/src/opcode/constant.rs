use super::{InstructionError, InstructionSuccess};
use crate::class_manager::{ClassManager, LoadedClass};
use crate::constant_pool::ConstantPoolEntry;
use crate::thread::Slot;
use crate::thread::Thread;
use crate::xconst_i;

pub fn nop(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    Ok(InstructionSuccess::Next(1))
}

xconst_i!(iconst_m1, Int, -1);
xconst_i!(iconst_0, Int, 0);
xconst_i!(iconst_1, Int, 1);
xconst_i!(iconst_2, Int, 2);
xconst_i!(iconst_3, Int, 3);
xconst_i!(iconst_4, Int, 4);
xconst_i!(iconst_5, Int, 5);

xconst_i!(lconst_0, Long, 0);
xconst_i!(lconst_1, Long, 1);

xconst_i!(fconst_0, Float, 0.0);
xconst_i!(fconst_1, Float, 1.0);
xconst_i!(fconst_2, Float, 2.0);

xconst_i!(dconst_0, Double, 0.0);
xconst_i!(dconst_1, Double, 1.0);

/// `aconst_null` pushes a null reference onto the stack.
pub fn aconst_null(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    frame.operand_stack.push(Slot::UndefinedReference);
    Ok(InstructionSuccess::Next(1))
}

/// `bipush` pushes a byte onto the stack as an integer.
pub fn bipush(thread: &mut Thread, value: i8) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    frame.operand_stack.push(Slot::Int(value as i32));
    Ok(InstructionSuccess::Next(2))
}

/// `sipush` pushes a short onto the stack as an integer.
pub fn sipush(thread: &mut Thread, value: i16) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    frame.operand_stack.push(Slot::Int(value as i32));
    Ok(InstructionSuccess::Next(3))
}

/// `ldc` pushes a constant from the constant pool onto the stack.
pub fn ldc(
    thread: &mut Thread,
    cm: &mut ClassManager,
    value: u8,
) -> Result<InstructionSuccess, InstructionError> {
    let pc = thread.pc;
    let frame = thread.current_frame_mut().unwrap();
    let class = frame.class;
    let LoadedClass::Loaded(class) = cm.get_class_by_id(class).unwrap() else {
        return Err(InstructionError::InvalidState {
            context: "Current class is not loaded!?".into(),
        });
    };
    let constant = class.constant_pool.get(value as usize).unwrap();
    match constant {
        ConstantPoolEntry::IntegerConstant(value) => {
            frame.operand_stack.push(Slot::Int(*value));
        }
        ConstantPoolEntry::FloatConstant(value) => {
            frame.operand_stack.push(Slot::Float(*value));
        }
        ConstantPoolEntry::ClassReference(value) => {
            let class_obj = cm.get_class_object(&value.clone()).unwrap();
            frame.operand_stack.push(Slot::ObjectReference(class_obj));
        }
        // TODO: Implement String reference and Class reference.
        _ => {
            log::error!(
                "ldc - invalid constant pool - running class {}, method {}, pc {}",
                class.name,
                frame.method,
                pc
            );
            return Err(InstructionError::InvalidState {
                context: format!("Invalid constant pool entry at {}: {:?}", value, constant),
            });
        }
    }
    Ok(InstructionSuccess::Next(2))
}

/// `ldc_w` pushes a constant from the constant pool onto the stack.
pub fn ldc_w(
    thread: &mut Thread,
    cm: &mut ClassManager,
    value: u16,
) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let class = frame.class;
    let LoadedClass::Loaded(class) = cm.get_class_by_id(class).unwrap() else {
        return Err(InstructionError::InvalidState {
            context: "Current class is not loaded!?".into(),
        });
    };
    let constant = class.constant_pool.get(value as usize).unwrap();

    match constant {
        ConstantPoolEntry::IntegerConstant(value) => {
            frame.operand_stack.push(Slot::Int(*value));
        }
        ConstantPoolEntry::FloatConstant(value) => {
            frame.operand_stack.push(Slot::Float(*value));
        }
        // TODO: Implement String reference and Class reference.
        _ => {
            return Err(InstructionError::InvalidState {
                context: format!("Invalid constant pool entry at {}: {:?}", value, constant),
            });
        }
    }
    Ok(InstructionSuccess::Next(3))
}

/// `ldc2_w` pushes a long/double constant from the constant pool onto the stack.
pub fn ldc2_w(
    thread: &mut Thread,
    cm: &mut ClassManager,
    value: u16,
) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let class = frame.class;
    let LoadedClass::Loaded(class) = cm.get_class_by_id(class).unwrap() else {
        return Err(InstructionError::InvalidState {
            context: "Current class is not loaded!?".into(),
        });
    };
    let constant = class.constant_pool.get(value as usize).unwrap();

    match constant {
        ConstantPoolEntry::LongConstant(value) => {
            frame.operand_stack.push(Slot::Long(*value));
        }
        ConstantPoolEntry::DoubleConstant(value) => {
            frame.operand_stack.push(Slot::Double(*value));
        }
        ConstantPoolEntry::ClassReference(value) => {
            let class_obj = cm.get_class_object(&value.clone()).unwrap();
            frame.operand_stack.push(Slot::ObjectReference(class_obj));
        }
        // TODO: Implement dynamic reference.
        _ => {
            return Err(InstructionError::InvalidState {
                context: format!("Invalid constant pool entry at {}: {:?}", value, constant),
            });
        }
    }
    Ok(InstructionSuccess::Next(3))
}

mod macros {
    #[macro_export]
    macro_rules! xconst_i {
        ($name:ident, $sloty:ident, $value:expr) => {
            /// Push a constant value onto the stack.
            pub fn $name(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
                let frame = thread.current_frame_mut().unwrap();
                frame.operand_stack.push(Slot::$sloty($value));
                Ok(InstructionSuccess::Next(1))
            }
        };
    }
}
