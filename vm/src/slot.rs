use dumpster::Collectable;
use dumpster::sync::Gc;
use reader::descriptor::{ArrayType, BaseType, FieldDescriptor, FieldType};

use crate::{alloc::{Array, ArrayRef, ObjectRef}, class::ConstantValue};

#[derive(Debug, Clone, Collectable)]
pub enum Slot {
    /// Like the constant pool, long and double entries take two slots.
    /// Hence the stucture representing the 2nd part of such entry.
    ///
    /// Note: This only applies to the local variables, not the operand stack.
    Tombstone,
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ReturnAddress(u32),
    /// This item is used to know the new instruction when returning from a method.
    ///
    /// It is an internal implementation detail and should not be used by the user.
    InvokationReturnAddress(u32),
    ArrayReference(ArrayRef),
    ObjectReference(ObjectRef),
    /// Basically the Null Pointer representation
    UndefinedReference,
}

impl Slot {
    pub fn size(&self) -> usize {
        match self {
            Slot::Tombstone => 0,
            Slot::Int(_)
            | Slot::Float(_)
            | Slot::ReturnAddress(_)
            | Slot::InvokationReturnAddress(_)
            | Slot::ArrayReference(_)
            | Slot::ObjectReference(_)
            | Slot::UndefinedReference => 1,
            Slot::Long(_) | Slot::Double(_) => 2,
        }
    }

    pub fn default_for(td: &FieldType) -> Slot {
        match td {
            &FieldType::BaseType(BaseType::Byte)
            | &FieldType::BaseType(BaseType::Char)
            | &FieldType::BaseType(BaseType::Short)
            | &FieldType::BaseType(BaseType::Int)
            | &FieldType::BaseType(BaseType::Boolean) => Slot::Int(0),
            &FieldType::BaseType(BaseType::Float) => Slot::Float(0.0),
            &FieldType::BaseType(BaseType::Long) => Slot::Long(0),
            &FieldType::BaseType(BaseType::Double) => Slot::Double(0.0),
            &FieldType::ObjectType(_)
            | &FieldType::ArrayType(_) => Slot::UndefinedReference,
        }
    }
}

impl From<ConstantValue> for Slot {
    fn from(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Integer(value) => Slot::Int(value),
            ConstantValue::Long(value) => Slot::Long(value),
            ConstantValue::Float(value) => Slot::Float(value),
            ConstantValue::Double(value) => Slot::Double(value),
        }
    }
}
