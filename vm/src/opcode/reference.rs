use super::{InstructionError, InstructionSuccess};
use crate::class::{Class, ClassId, Method};
use crate::class_manager::{ClassManager, LoadedClass, LoadingClass};
use crate::constant_pool::ConstantPoolEntry;
use crate::thread::{Frame, Slot, Thread};

/// `getstatic` gets a static field value of a class, where the field is identified
///  by field reference in the constant pool index.
pub fn getstatic(
    thread: &mut Thread,
    cm: &mut ClassManager,
    index: u16,
) -> Result<InstructionSuccess, InstructionError> {
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
    }) = class.constant_pool.get_field_ref(index as usize).cloned()
    else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "FieldRef not found: ClassId({}), constant pool index {}",
                class.id.0, index
            ),
        });
    };
    cm.request_class_load(implementor.clone());
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
                "Field not found: ClassId({}), field name {}, field descriptor {:?}",
                implementor.0, field_name, field_descriptor
            ),
        });
    };

    if !field.is_static() {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Field is not static: ClassId({}), field name {}, field descriptor {:?}",
                implementor.0, field_name, field_descriptor
            ),
        });
    }

    let Some(value) = field.get_value() else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Field not initialized: ClassId({}), field index {}",
                implementor.0, index
            ),
        });
    };
    frame.operand_stack.push(value.clone());
    Ok(InstructionSuccess::Next(3))
}

/// `putstatic` sets static field to a value in a class, where the field is identified
/// by field reference in the constant pool index.
pub fn putstatic(
    thread: &mut Thread,
    cm: &mut ClassManager,
    index: u16,
) -> Result<InstructionSuccess, InstructionError> {
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
    cm.request_class_load(implementor.clone());
    let Some(LoadedClass::Loaded(impl_class)) = cm.get_mut_class_by_id(implementor.clone()) else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Implementor class not found / not initialized: ClassId({})",
                implementor.0
            ),
        });
    };

    let class_initialized =
        impl_class.initialized.get().is_some() && impl_class.initialized.get().cloned().unwrap();

    let Some(field) = impl_class.get_mut_field(&field_name) else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Field not found: ClassId({}), field name {}, field descriptor {:?}",
                implementor.0, field_name, field_descriptor
            ),
        });
    };

    if !field.is_static() {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Field is not static: ClassId({}), field name {}, field descriptor {:?}",
                implementor.0, field_name, field_descriptor
            ),
        });
    }

    if field.is_final() && class_initialized {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Field is final and class is already initialized: ClassId({}), field name {}, field descriptor {:?}",
                implementor.0, field_name, field_descriptor
            ),
        });
    }

    let Some(value) = frame.operand_stack.pop() else {
        return Err(InstructionError::InvalidState {
            context: format!("Operand stack is empty"),
        });
    };
    field.value = value;
    Ok(InstructionSuccess::Next(3))
}

/// `invokestatic` invokes a static method and puts the result on the operand stack.
pub fn invokestatic(
    thread: &mut Thread,
    cm: &mut ClassManager,
    index: u16,
) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let (method_name, method_descriptor, implementor) = {
        let Some(LoadedClass::Loaded(class)) = cm.get_class_by_id(frame.class) else {
            return Err(InstructionError::InvalidState {
                context: format!(
                    "Class not found (or not loaded): ClassId({})",
                    frame.class.0
                ),
            });
        };

        let Some(ConstantPoolEntry::MethodReference {
            method_name,
            method_descriptor,
            implementor,
        }) = class.constant_pool.get_method_ref(index as usize).cloned()
        else {
            return Err(InstructionError::InvalidState {
                context: format!(
                    "MethodRef not found: ClassId({}), constant pool index {}",
                    class.id.0, index
                ),
            });
        };

        (method_name, method_descriptor, implementor)
    };

    cm.request_class_load(implementor.clone());
    let Some(LoadedClass::Loaded(impl_class)) = cm.get_class_by_id(implementor) else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Implementor class not found / not initialized: ClassId({})",
                implementor.0
            ),
        });
    };

    let Some((method_id, method)) = impl_class.get_method(&method_name, &method_descriptor) else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Method not found: ClassId({}), method name {}, method descriptor {:?}",
                implementor.0, method_name, method_descriptor
            ),
        });
    };

    let mut args = Vec::new();
    for _ in 0..method_descriptor.args_count() {
        let arg = frame.operand_stack.pop().unwrap();
        args.push(arg);
    }

    if !method.is_static() {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Method is not static: ClassId({}), method name {}, method descriptor {:?}",
                implementor.0, method_name, method_descriptor
            ),
        });
    }

    invoke(thread, cm, implementor, method_id, args, 3)
}

fn invoke(
    thread: &mut Thread,
    cm: &mut ClassManager,
    class_id: ClassId,
    method_id: usize,
    args: Vec<Slot>,
    next_instruction: usize,
) -> Result<InstructionSuccess, InstructionError> {
    let Some(LoadedClass::Loaded(impl_class)) = cm.get_class_by_id(class_id) else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Implementor class not found / not initialized: ClassId({})",
                class_id.0
            ),
        });
    };

    let Some(method) = impl_class.get_method_by_index(method_id) else {
        return Err(InstructionError::InvalidState {
            context: format!(
                "Method not found: ClassId({}), method index {}",
                class_id.0, method_id
            ),
        });
    };

    if method.is_native() {
        log::debug!(
            "Call to native method: {}::{}, {:?}, with args:\n{:?}",
            impl_class.name,
            method.name,
            method.descriptor,
            args
        );
        log::warn!("Native methods are not implemented yet, skipping the invokation");
        Ok(InstructionSuccess::Next(next_instruction))
    } else {
        let code = method
            .get_code()
            .expect("A non-native method has no code attribute, THIS IS WRONG!");
        let frame = Frame::new(class_id, method_id, code.max_locals as usize);

        // TODO: synchronized - implement monitorenter/monitorexit

        // Push the "return address" onto the stack
        let old_pc = thread.pc + next_instruction;

        let cur_frame = thread.current_frame_mut().unwrap();
        cur_frame
            .operand_stack
            .push(Slot::InvokationReturnAddress(old_pc as u32));

        // Push the new frame onto the stack, with the arguments in the local variables.
        thread.push_frame(frame);
        let frame = thread.current_frame_mut().unwrap();
        let mut arg_pos = 0;
        for arg in args.into_iter() {
            match arg {
                Slot::Int(_) | Slot::Float(_) => {
                    frame.local_variables[arg_pos] = arg;
                    arg_pos += 1;
                }
                Slot::Long(_) | Slot::Double(_) => {
                    frame.local_variables[arg_pos] = arg;
                    frame.local_variables[arg_pos + 1] = Slot::Tombstone;
                    arg_pos += 2;
                }
                _ => {
                    panic!("Invalid argument type");
                }
            }
        }
        Ok(InstructionSuccess::FrameChange(0))
    }
}
