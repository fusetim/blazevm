use super::InstructionError;
use crate::class_manager::{ClassManager, LoadedClass};
use crate::constant_pool::ConstantPoolEntry;
use crate::slot::Slot;
use crate::thread::Thread;

/// `getstatic` gets a static field value of a class, where the field is identified
///  by field reference in the constant pool index.
pub fn getstatic(
    thread: &mut Thread,
    cm: &mut ClassManager,
    index: u16,
) -> Result<(), InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let class = frame.class;
    let Some(LoadedClass::Loaded(class)) = cm.get_class_by_id(class) else {
        return Err(InstructionError::InvalidState {
            context: format!("Class not found: ClassId({})", class.0),
        });
    };
    let Some(ConstantPoolEntry::FieldReference {
        field_name,
        field_descriptor,
        implementor,
    }) = class.constant_pool.get_field_ref(index as usize)
    else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "FieldRef not found: ClassId({}), constant pool index {}",
                class.id.0, index
            ),
        });
    };
    let Some(LoadedClass::Loaded(impl_class)) = cm.get_class_by_id(implementor.clone()) else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Implementor class not found / not initialized: ClassId({})",
                implementor.0
            ),
        });
    };
    let Some(field) = impl_class.get_field(&field_name) else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Field not found: ClassId({}), field name {}, field descriptor {}",
                implementor.0, field_name, field_descriptor
            ),
        });
    };
    let Some(value) = field.get_value() else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Field not initialized: ClassId({}), field index {}",
                class.id.0, index
            ),
        });
    };
    frame.operand_stack.push(value.clone());
    thread.pc += 3;
    Ok(())
}

/// `putstatic` sets static field to a value in a class, where the field is identified
/// by field reference in the constant pool index.
pub fn putstatic(
    thread: &mut Thread,
    cm: &mut ClassManager,
    index: u16,
) -> Result<(), InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let (field_name, field_descriptor, implementor) = {
        let class = frame.class;
        let Some(LoadedClass::Loaded(class)) = cm.get_class_by_id(class) else {
            return Err(InstructionError::InvalidState {
                context: format!("Class not found: ClassId({})", class.0),
            });
        };
        let Some(ConstantPoolEntry::FieldReference {
            field_name,
            field_descriptor,
            implementor,
        }) = class.constant_pool.get_field_ref(index as usize)
        else {
            return Err(InstructionError::InvalidState {
                context: format!(
                    "FieldRef not found: ClassId({}), constant pool index {}",
                    class.id.0, index
                ),
            });
        };
        (
            field_name.clone(),
            field_descriptor.clone(),
            implementor.clone(),
        )
    };
    let Some(LoadedClass::Loaded(impl_class)) = cm.get_mut_class_by_id(implementor.clone()) else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Implementor class not found / not initialized: ClassId({})",
                implementor.0
            ),
        });
    };
    let Some(field) = impl_class.get_mut_field(&field_name) else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Field not found: ClassId({}), field name {}, field descriptor {}",
                implementor.0, field_name, field_descriptor
            ),
        });
    };
    // TODO: Check the field is actually STATIC and not FINAL (or else we should be in <clinit>)
    let Some(value) = frame.operand_stack.pop() else {
        return Err(InstructionError::InvalidState {
            context: format!("Operand stack is empty"),
        });
    };
    field.value = value;
    thread.pc += 3;
    Ok(())
}
