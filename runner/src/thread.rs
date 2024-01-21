use crate::{class::ClassId, class_loader, class_manager::{self, LoadedClass}};

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
            let code = method.get_code().expect("Code attribute not found, probably a native method");
            let instruction = code.instructions[self.pc];
            // TODO: Do something with the instruction
            self.pc += 1;
        }
    }

    fn push_frame(&mut self, frame: Frame) {
        self.stack.push(frame);
    }

    fn pop_frame(&mut self) -> Option<Frame> {
        self.stack.pop()
    }

    fn current_frame(&self) -> Option<&Frame> {
        self.stack.last()
    }

    fn current_frame_mut(&mut self) -> Option<&mut Frame> {
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

#[derive(Debug, Clone)]
pub enum Slot {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    // Object(ClassId),
}