use snafu::Snafu;

use crate::{
    class::ClassId,
    class_manager::{self, LoadedClass},
    opcode::InstructionSuccess,
};
use std::io::Cursor;

pub use crate::slot::Slot;

#[derive(Debug, Clone)]
pub struct Thread {
    pub pc: usize,
    pub stack: Vec<Frame>,
}

impl Thread {
    pub fn new() -> Self {
        Self {
            pc: 0,
            stack: vec![],
        }
    }

    pub fn execute(
        &mut self,
        class_manager: &mut class_manager::ClassManager,
    ) -> Result<(), ExecutionError> {
        while let Some(frame) = self.current_frame_mut() {
            let LoadedClass::Loaded(class) = class_manager.get_class_by_id(frame.class).unwrap()
            else {
                return Err(ExecutionError::ClassNotLoaded);
            };
            let Some(method) = class.get_method_by_index(frame.method) else {
                return Err(ExecutionError::MethodNotLoaded);
            };

            log::debug!("Executing method: {}#{}", class.name, method.name);
            log::debug!("Current local vars: {:?}", frame.local_variables);

            // TODO: Native methods
            let code = method
                .get_code()
                .expect("Code attribute not found, probably a native method");

            let mut inst_reader = Cursor::new(code.instructions.clone());
            loop {
                inst_reader.set_position(self.pc as u64);
                let inst = match crate::opcode::read_instruction(&mut inst_reader) {
                    Ok((_, inst)) => inst,
                    Err(e) => {
                        return Err(ExecutionError::InstructionParseError { source: e });
                    }
                };
                log::trace!(
                    "Executing instruction: {:?} with current stack: {:?}",
                    inst,
                    self.current_frame()
                );
                match crate::opcode::Opcode::execute(&inst, self, class_manager) {
                    Ok(InstructionSuccess::Next(n)) => {
                        self.pc += n;
                    }
                    Ok(InstructionSuccess::JumpRelative(offset)) => {
                        self.pc = ((self.pc as isize) + offset) as usize;
                    }
                    Ok(InstructionSuccess::JumpAbsolute(offset)) => {
                        self.pc = offset;
                    }
                    Ok(InstructionSuccess::FrameChange(pc)) => {
                        self.pc = pc;
                        break;
                    }
                    Ok(InstructionSuccess::Completed) => {
                        break;
                    }
                    Err(e) => {
                        return Err(ExecutionError::InstructionExecutionError { source: e });
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) fn push_frame(&mut self, frame: Frame) {
        self.stack.push(frame);
    }

    pub(crate) fn pop_frame(&mut self) -> Option<Frame> {
        self.stack.pop()
    }

    pub(crate) fn current_frame(&self) -> Option<&Frame> {
        self.stack.last()
    }

    pub(crate) fn current_frame_mut(&mut self) -> Option<&mut Frame> {
        self.stack.last_mut()
    }

    pub fn reset(&mut self) {
        self.pc = 0;
        self.stack.clear();
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub local_variables: Vec<Slot>,
    pub operand_stack: Vec<Slot>,
    pub class: ClassId,
    pub method: usize,
}

impl Frame {
    pub fn new(class: ClassId, method: usize, varlen: usize) -> Self {
        Self {
            local_variables: vec![Slot::Tombstone; varlen],
            operand_stack: vec![],
            class,
            method,
        }
    }

    pub fn get_local_variable(&self, index: usize) -> Option<&Slot> {
        self.local_variables.get(index)
    }

    pub fn get_local_variable_mut(&mut self, index: usize) -> Option<&mut Slot> {
        self.local_variables.get_mut(index)
    }

    pub fn set_local_variable(&mut self, index: usize, value: Slot) {
        self.local_variables[index] = value;
    }
}

/// Errors that can occur during execution of a thread
#[derive(Debug, Snafu)]
pub enum ExecutionError {
    /// The current class is not loaded
    #[snafu(display("Class not loaded"))]
    ClassNotLoaded,

    /// The current method is not loaded
    #[snafu(display("Method not loaded"))]
    MethodNotLoaded,

    /// Impossible to parse the current instruction
    #[snafu(display("Error parsing instruction, source: {}", source))]
    InstructionParseError {
        source: crate::opcode::InstructionError,
    },

    /// Error occured during execution of an instruction
    #[snafu(display("Error executing instruction, source: {}", source))]
    InstructionExecutionError {
        source: crate::opcode::InstructionError,
    },
}
