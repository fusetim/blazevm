use crate::{class::ClassId, from_item_array, item_array};
use dumpster::{sync::Gc, Collectable};
use reader::descriptor::{ArrayType, BaseType, FieldType, ObjectType};
use std::sync::RwLock;

use super::ObjectRef;

/// Garbage collected array reference
pub type ArrayRef = Gc<Array>;

/// JVM representation of an array
#[derive(Debug, Collectable)]
pub enum Array {
    Int(IntArray),
    Long(LongArray),
    Float(FloatArray),
    Double(DoubleArray),
    Byte(ByteArray),
    Char(CharArray),
    Short(ShortArray),
    ObjectRef(ObjectRefArray),
    ArrayRef(ArrayRefArray),
    Boolean(BoolArray),
}

item_array!(IntArray, i32, 0);
item_array!(LongArray, i64, 0);
item_array!(FloatArray, f32, 0.0);
item_array!(DoubleArray, f64, 0.0);
item_array!(ByteArray, i8, 0);
item_array!(BoolArray, bool, false);
item_array!(CharArray, u16, 0);
item_array!(ShortArray, i16, 0);

from_item_array!(Int, IntArray);
from_item_array!(Long, LongArray);
from_item_array!(Float, FloatArray);
from_item_array!(Double, DoubleArray);
from_item_array!(Byte, ByteArray);
from_item_array!(Boolean, BoolArray);
from_item_array!(Char, CharArray);
from_item_array!(Short, ShortArray);
from_item_array!(ArrayRef, ArrayRefArray);
from_item_array!(ObjectRef, ObjectRefArray);

impl Array {
    /// Get the length of the array.
    pub fn len(&self) -> usize {
        match self {
            Array::Int(array) => array.len(),
            Array::Long(array) => array.len(),
            Array::Float(array) => array.len(),
            Array::Double(array) => array.len(),
            Array::Byte(array) => array.len(),
            Array::Boolean(array) => array.len(),
            Array::Char(array) => array.len(),
            Array::Short(array) => array.len(),
            Array::ObjectRef(array) => array.len(),
            Array::ArrayRef(array) => array.len(),
        }
    }
}

#[derive(Debug, Collectable)]
pub struct ObjectRefArray {
    pub class_id: ClassId,
    pub data: RwLock<Vec<Option<ObjectRef>>>,
}

impl ObjectRefArray {
    /// Create a new array of object of the given size and type.
    pub fn new(class_id: ClassId, size: usize) -> Self {
        Self {
            class_id,
            data: RwLock::new(vec![None; size]),
        }
    }

    /// Get the object at the given index
    pub fn get(&self, index: usize) -> Option<Option<ObjectRef>> {
        self.data
            .read()
            .expect("rwlock has been poisoned, cannot get a ref to array element")
            .get(index)
            .cloned()
    }

    /// Set the object at the given index
    pub fn set(&self, index: usize, value: Option<ObjectRef>) {
        self.data
            .write()
            .expect("rwlock has been poisoned, cannot get a mutable ref to array element")[index] =
            value;
    }

    /// Get the length of the array
    pub fn len(&self) -> usize {
        self.data
            .read()
            .expect("rwlock has been poisoned, cannot get length to array element")
            .len()
    }

    /// Get the class id of the array
    pub fn class_id(&self) -> ClassId {
        self.class_id
    }
}

#[derive(Debug, Collectable)]
pub struct ArrayRefArray {
    pub item_ty: ArrayType,
    pub data: RwLock<Vec<Option<ArrayRef>>>,
}

impl ArrayRefArray {
    /// Create a new array of array of the given size and type.
    pub fn new(item_ty: ArrayType, size: usize) -> Self {
        Self {
            item_ty,
            data: RwLock::new(vec![None; size]),
        }
    }

    /// Get the array at the given index
    pub fn get(&self, index: usize) -> Option<Option<ArrayRef>> {
        self.data
            .read()
            .expect("rwlock has been poisoned, cannot get a ref to array element")
            .get(index)
            .cloned()
    }

    /// Set the array at the given index
    pub fn set(&self, index: usize, value: Option<ArrayRef>) {
        self.data
            .write()
            .expect("rwlock has been poisoned, cannot get a mutable ref to array element")[index] =
            value;
    }

    /// Get the length of the array
    pub fn len(&self) -> usize {
        self.data
            .read()
            .expect("rwlock has been poisoned, cannot get length to array element")
            .len()
    }

    /// Get the class id of the array
    pub fn item_type(&self) -> &ArrayType {
        &self.item_ty
    }
}

impl CharArray {
    /// Create a Char Array from a rust string
    pub fn from_string(string: &str) -> Self {
        let data = string.encode_utf16().collect();
        Self {
            data: RwLock::new(data),
        }
    }
}

mod macros {
    #[macro_export]
    macro_rules! item_array {
        ($name:ident, $ty:ty, $default_value:expr) => {
            /// JVM representation of an array of such type
            #[derive(Debug, Collectable)]
            pub struct $name {
                pub data: RwLock<Vec<$ty>>,
            }

            impl $name {
                /// Create a new array of the given size
                pub fn new(size: usize) -> Self {
                    Self {
                        data: RwLock::new(vec![$default_value; size]),
                    }
                }

                /// Get the value at the given index
                pub fn get(&self, index: usize) -> Option<$ty> {
                    self.data
                        .read()
                        .expect("rwlock has been poisoned, cannot get a ref to array element")
                        .get(index)
                        .cloned()
                }

                /// Set the value at the given index
                ///
                /// # Panics
                /// Panics if the index is out of bounds
                pub fn set(&self, index: usize, value: $ty) {
                    self.data.write().expect(
                        "rwlock has been poisoned, cannot get a mutable ref to array element",
                    )[index] = value;
                }

                /// Get the length of the array
                pub fn len(&self) -> usize {
                    self.data
                        .read()
                        .expect("rwlock has been poisoned, cannot get length to array element")
                        .len()
                }
            }

            impl From<Vec<$ty>> for $name {
                fn from(data: Vec<$ty>) -> Self {
                    Self {
                        data: RwLock::new(data),
                    }
                }
            }

            impl From<$name> for Vec<$ty> {
                fn from(array: $name) -> Self {
                    array.data.into_inner().expect(
                        "rwlock has been poisoned, cannot consume it to access the array data",
                    )
                }
            }
        };
    }

    #[macro_export]
    macro_rules! from_item_array {
        ($name:ident, $ty:ty) => {
            impl From<$ty> for Array {
                fn from(array: $ty) -> Self {
                    Array::$name(array)
                }
            }
        };
    }
}
