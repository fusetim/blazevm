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

    pub fn execute(&mut self, class_manager: &mut class_manager::ClassManager) {
        while let Some(frame) = self.current_frame_mut() {
            let LoadedClass::Loaded(class) = class_manager.get_class_by_id(frame.class).unwrap()
            else {
                panic!("Class not found");
            };
            let method = class.get_method_by_index(frame.method).unwrap();
            // TODO: Native methods
            let code = method
                .get_code()
                .expect("Code attribute not found, probably a native method");

            let mut inst_reader = Cursor::new(&code.instructions);
            loop {
                inst_reader.set_position(self.pc as u64);
                let inst = match crate::opcode::read_instruction(&mut inst_reader) {
                    Ok((_, inst)) => inst,
                    Err(e) => {
                        panic!("Error reading instruction: {:?}", e);
                    }
                };
                match crate::opcode::Opcode::execute(&inst, class_manager, class_manager) {
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
                        panic!("Error executing instruction: {:?}", e);
                    }
                }
            }
        }
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
