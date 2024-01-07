/// Runtime representation of the constant pool.
#[derive(Debug)]
pub struct RTConstantPool {
    pub entries: Vec<RTConstantPoolEntry>,
}

impl RTConstantPool {
    pub fn new(entries: Vec<RTConstantPoolEntry>) -> Self {
        Self { entries }
    }

    pub fn get(&self, index: usize) -> Option<&RTConstantPoolEntry> {
        self.entries.get(index)
    }

    pub fn append(&mut self, entry: RTConstantPoolEntry) {
        self.entries.push(entry)
    }
}

/// Runtime representation of a constant pool entry.
#[derive(Debug)]
pub enum RTConstantPoolEntry {
    IntegerConstant(i32),
    FloatConstant(f32),
    LongConstant(i64),
    DoubleConstant(f64),
    // TODO: String constant should be a reference to a java String object.
    StringConstant(String),
    // TODO: Implement the rest of the constant pool entries, in particular
    // the symbolic references (class, field, method, interface method, ...).
}
