use dumpster::Collectable;

/// Runtime representation of the constant pool.
#[derive(Debug, Collectable)]
pub struct ConstantPool {
    pub entries: Vec<ConstantPoolEntry>,
}

impl ConstantPool {
    pub fn new(entries: Vec<ConstantPoolEntry>) -> Self {
        Self { entries }
    }

    pub fn get(&self, index: usize) -> Option<&ConstantPoolEntry> {
        self.entries.get(index)
    }

    pub fn append(&mut self, entry: ConstantPoolEntry) {
        self.entries.push(entry)
    }
}

/// Runtime representation of a constant pool entry.
#[derive(Debug, Collectable)]
pub enum ConstantPoolEntry {
    IntegerConstant(i32),
    FloatConstant(f32),
    LongConstant(i64),
    DoubleConstant(f64),
    // TODO: String constant should be a reference to a java String object.
    StringConstant(String),
    // TODO: Implement the rest of the constant pool entries, in particular
    // the symbolic references (class, field, method, interface method, ...).
}
