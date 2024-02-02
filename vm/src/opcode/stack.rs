use super::{InstructionError, InstructionSuccess};
use crate::thread::Slot;
use crate::thread::Thread;

/// `pop` pops the top operand stack value.
///
/// Note: If the top value is a long or double, it is treated as two values.
/// The pop instruction MUST NOT be used to pop a value that is a part of a
/// double-width operand.
pub fn pop(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    match frame.operand_stack.pop() {
        Some(Slot::Double(_)) | Some(Slot::Long(_)) => Err(InstructionError::InvalidState {
            context: "Illegal operation, pop on stack where top of stack is a long/double slot."
                .into(),
        }),
        Some(_) => Ok(InstructionSuccess::Next(1)),
        None => Err(InstructionError::InvalidState {
            context: "Operand stack is empty".into(),
        }),
    }
}

/// `pop2` pops the top one or two operand stack values.
///
/// Note: If the top value is a long or double, it is treated as two values.
/// Otherwise, pop2 removes two single-word values from the operand stack.
pub fn pop2(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    match frame.operand_stack.pop() {
        Some(Slot::Double(_)) | Some(Slot::Long(_)) => Ok(InstructionSuccess::Next(1)),
        Some(_) => match frame.operand_stack.pop() {
            Some(Slot::Double(_)) | Some(Slot::Long(_)) => Ok(InstructionSuccess::Next(1)),
            Some(_) => Err(InstructionError::InvalidState {
                context:
                    "Illegal operation, pop2 on stack where top of stack are long/double slots."
                        .into(),
            }),
            None => Err(InstructionError::InvalidState {
                context: "Operand stack is len 1, pop2 cannot remove two elements.".into(),
            }),
        },
        None => Err(InstructionError::InvalidState {
            context: "Operand stack is empty".into(),
        }),
    }
}

/// `dup` duplicates the top operand stack value.
///
/// Note: Must only be used on a single-word value.
pub fn dup(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    match frame.operand_stack.last() {
        Some(Slot::Double(_)) | Some(Slot::Long(_)) => Err(InstructionError::InvalidState {
            context: "Illegal operation, dup on stack where top of stack is a long/double slot."
                .into(),
        }),
        Some(slot) => {
            frame.operand_stack.push(slot.clone());
            Ok(InstructionSuccess::Next(1))
        }
        None => Err(InstructionError::InvalidState {
            context: "Operand stack is empty".into(),
        }),
    }
}

/// `dup_x1` duplicates the top operand stack value and inserts two values down.
///
/// Note: Must only be used on a single-word value.
pub fn dup_x1(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    match frame.operand_stack.last() {
        Some(Slot::Double(_)) | Some(Slot::Long(_)) => Err(InstructionError::InvalidState {
            context: "Illegal operation, dup_x1 on stack where top of stack is a long/double slot."
                .into(),
        }),
        Some(slot) => {
            let slot = slot.clone();
            frame.operand_stack.pop();
            match frame.operand_stack.last() {
                Some(Slot::Double(_)) | Some(Slot::Long(_)) => {
                    Err(InstructionError::InvalidState { context: "Illegal operation, dup_x1 on stack where second slot is a long/double slot.".into() })
                }
                Some(_) => {
                    let slot2 = frame.operand_stack.pop().unwrap();
                    frame.operand_stack.push(slot.clone());
                    frame.operand_stack.push(slot2);
                    frame.operand_stack.push(slot);
                    Ok(InstructionSuccess::Next(1))
                }
                None => {
                    Err(InstructionError::InvalidState { context: "Operand stack is empty".into() })
                }
            }
        }
        None => Err(InstructionError::InvalidState {
            context: "Operand stack is empty".into(),
        }),
    }
}

/// `dup_x2` duplicates the top operand stack value and inserts two or three values down.
///
/// Note: Must only be used on a single-word value, but is practical when the 2nd value is
/// a long or double.
pub fn dup_x2(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    match frame.operand_stack.last() {
        Some(Slot::Double(_)) | Some(Slot::Long(_)) => Err(InstructionError::InvalidState {
            context: "Illegal operation, dup_x2 on stack where top of stack is a long/double slot."
                .into(),
        }),
        Some(slot) => {
            let slot = slot.clone();
            frame.operand_stack.pop();
            match frame.operand_stack.last() {
                Some(Slot::Double(_)) | Some(Slot::Long(_)) => {
                    let slot2 = frame.operand_stack.pop().unwrap();
                    frame.operand_stack.push(slot.clone());
                    frame.operand_stack.push(slot2);
                    frame.operand_stack.push(slot);
                    Ok(InstructionSuccess::Next(1))
                }
                Some(_) => {
                    let slot2 = frame.operand_stack.pop().unwrap();
                    frame.operand_stack.push(slot.clone());
                    match frame.operand_stack.last() {
                        Some(Slot::Double(_)) | Some(Slot::Long(_)) => {
                            Err(InstructionError::InvalidState { context: "Illegal operation, dup_x2 on stack where third slot is a long/double slot.".into() })
                        }
                        Some(_) => {
                            let slot3 = frame.operand_stack.pop().unwrap();
                            frame.operand_stack.push(slot.clone());
                            frame.operand_stack.push(slot3);
                            frame.operand_stack.push(slot2);
                            frame.operand_stack.push(slot);
                            Ok(InstructionSuccess::Next(1))
                        }
                        None => {
                            Err(InstructionError::InvalidState { context: "Operand stack is empty".into() })
                        }
                    }
                }
                None => Err(InstructionError::InvalidState {
                    context: "Operand stack is empty".into(),
                }),
            }
        }
        None => Err(InstructionError::InvalidState {
            context: "Operand stack is empty".into(),
        }),
    }
}

/// `dup2` duplicates the top one or two operand stack values.
pub fn dup2(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    match frame.operand_stack.last() {
        // If 1st slot is a long or double, it is treated as two values.
        Some(Slot::Double(_)) | Some(Slot::Long(_)) => {
            let slot = frame.operand_stack.pop().unwrap();
            frame.operand_stack.push(slot.clone());
            frame.operand_stack.push(slot);
            Ok(InstructionSuccess::Next(1))
        }
        Some(_) => {
            // Otherwise, dup the two single-word values from the operand stack.
            let slot1 = frame.operand_stack.pop().unwrap();
            match frame.operand_stack.last() {
                Some(Slot::Double(_)) | Some(Slot::Long(_)) => {
                    Err(InstructionError::InvalidState { context: "Illegal operation, dup2 on stack where second slot is a long/double slot.".into() })
                }
                Some(_) => {
                    let slot2 = frame.operand_stack.pop().unwrap();
                    frame.operand_stack.push(slot2.clone());
                    frame.operand_stack.push(slot1.clone());
                    frame.operand_stack.push(slot2.clone());
                    frame.operand_stack.push(slot1.clone());
                    Ok(InstructionSuccess::Next(1))
                }
                None => {
                    Err(InstructionError::InvalidState { context: "Operand stack is empty".into() })
                }
            }
        }
        None => Err(InstructionError::InvalidState {
            context: "Operand stack is empty".into(),
        }),
    }
}

/// `dup2_x1` duplicates the top one or two operand stack values and inserts two or three values down.
pub fn dup2_x1(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let len = frame.operand_stack.len();
    if len < 2 {
        return Err(InstructionError::InvalidState {
            context: "Operand stack is empty".into(),
        });
    }
    if frame.operand_stack[len - 1].size() == 1 {
        if len > 2
            && frame.operand_stack[len - 2].size() == 1
            && frame.operand_stack[len - 3].size() == 1
        {
            let slot1 = frame.operand_stack.pop().unwrap();
            let slot2 = frame.operand_stack.pop().unwrap();
            let slot3 = frame.operand_stack.pop().unwrap();
            frame.operand_stack.push(slot2.clone());
            frame.operand_stack.push(slot1.clone());
            frame.operand_stack.push(slot3.clone());
            frame.operand_stack.push(slot2.clone());
            frame.operand_stack.push(slot1.clone());
        } else {
            return Err(InstructionError::InvalidState { context: "Illegal operation, dup2_x1 on stack where 2nd/3rd value on stack is a long/double slot.".into() });
        }
    } else if frame.operand_stack[len - 2].size() == 1 {
        let slot1 = frame.operand_stack.pop().unwrap();
        let slot2 = frame.operand_stack.pop().unwrap();
        frame.operand_stack.push(slot1.clone());
        frame.operand_stack.push(slot2.clone());
        frame.operand_stack.push(slot1.clone());
    } else {
        return Err(InstructionError::InvalidState {
            context:
                "Illegal operation, dup2_x1 on stack where top of stack is a long/double slot."
                    .into(),
        });
    }
    Ok(InstructionSuccess::Next(1))
}

/// `dup2_x2` duplicates the top one or two operand stack values and inserts two, three, or four values down.
pub fn dup2_x2(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let len = frame.operand_stack.len();
    if len < 2 {
        return Err(InstructionError::InvalidState {
            context: "Operand stack is empty".into(),
        });
    }
    if frame.operand_stack[len - 1].size() == 1 {
        if frame.operand_stack[len - 2].size() == 1 {
            // Form 1 or 3
            if len > 3
                && frame.operand_stack[len - 3].size() == 1
                && frame.operand_stack[len - 4].size() == 1
            {
                // Form 1
                let slot1 = frame.operand_stack.pop().unwrap();
                let slot2 = frame.operand_stack.pop().unwrap();
                let slot3 = frame.operand_stack.pop().unwrap();
                let slot4 = frame.operand_stack.pop().unwrap();
                frame.operand_stack.push(slot2.clone());
                frame.operand_stack.push(slot1.clone());
                frame.operand_stack.push(slot4.clone());
                frame.operand_stack.push(slot3.clone());
                frame.operand_stack.push(slot2.clone());
                frame.operand_stack.push(slot1.clone());
            } else if len > 2 && frame.operand_stack[len - 3].size() == 2 {
                // Form 3
                let slot1 = frame.operand_stack.pop().unwrap();
                let slot2 = frame.operand_stack.pop().unwrap();
                let slot3 = frame.operand_stack.pop().unwrap();
                frame.operand_stack.push(slot2.clone());
                frame.operand_stack.push(slot1.clone());
                frame.operand_stack.push(slot3.clone());
                frame.operand_stack.push(slot2.clone());
                frame.operand_stack.push(slot1.clone());
            } else {
                return Err(InstructionError::InvalidState { context: "Illegal operation, dup2_x2 on stack where 3rd/4th value on stack is a long/double slot.".into() });
            }
        } else {
            return Err(InstructionError::InvalidState { context: "Illegal operation, dup2_x2 on stack where 3rd value on stack is a long/double slot.".into() });
        }
    } else if frame.operand_stack[len - 2].size() == 1 {
        // Form 2
        if len > 2 && frame.operand_stack[len - 3].size() == 1 {
            let slot1 = frame.operand_stack.pop().unwrap();
            let slot2 = frame.operand_stack.pop().unwrap();
            let slot3 = frame.operand_stack.pop().unwrap();
            frame.operand_stack.push(slot1.clone());
            frame.operand_stack.push(slot3.clone());
            frame.operand_stack.push(slot2.clone());
            frame.operand_stack.push(slot1.clone());
        } else {
            return Err(InstructionError::InvalidState { context: "Illegal operation, dup2_x2 on stack where 3rd value on stack is a long/double slot.".into() });
        }
    } else {
        // Form 4
        let slot1 = frame.operand_stack.pop().unwrap();
        let slot2 = frame.operand_stack.pop().unwrap();
        frame.operand_stack.push(slot1.clone());
        frame.operand_stack.push(slot2.clone());
        frame.operand_stack.push(slot1.clone());
    }
    Ok(InstructionSuccess::Next(1))
}

/// `swap` swaps the top two operand stack values.
///
/// Note: Must only be used on single-word values.
pub fn swap(thread: &mut Thread) -> Result<InstructionSuccess, InstructionError> {
    let frame = thread.current_frame_mut().unwrap();
    let len = frame.operand_stack.len();
    if len < 2 {
        return Err(InstructionError::InvalidState {
            context: "Operand stack is empty".into(),
        });
    }
    if frame.operand_stack[len - 1].size() == 1 && frame.operand_stack[len - 2].size() == 1 {
        let slot1 = frame.operand_stack.pop().unwrap();
        let slot2 = frame.operand_stack.pop().unwrap();
        frame.operand_stack.push(slot1.clone());
        frame.operand_stack.push(slot2.clone());
        Ok(InstructionSuccess::Next(1))
    } else {
        Err(InstructionError::InvalidState {
            context:
                "Illegal operation, swap on stack where top of the stack is a long/double slot."
                    .into(),
        })
    }
}
