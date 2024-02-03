use dumpster::{sync::Gc, Collectable};
use crate::item_array;
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
}

item_array!(IntArray,    i32, 0);
item_array!(LongArray,   i64, 0);
item_array!(FloatArray,  f32, 0.0);
item_array!(DoubleArray, f64, 0.0);
item_array!(ByteArray,    i8, 0);
item_array!(CharArray,   u16, 0);
item_array!(ShortArray,  i16, 0);
item_array!(ArrayRefArray, Option<ArrayRef>, None);
item_array!(ObjectRefArray, Option<ObjectRef>, None);


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
                    Self { data: RwLock::new(vec![$default_value; size]) }
                }

                /// Get the value at the given index
                pub fn get(&self, index: usize) -> Option<$ty> {
                    self.data.read().expect("rwlock has been poisoned, cannot get a ref to array element").get(index).cloned()
                }

                /// Set the value at the given index
                ///
                /// # Panics
                /// Panics if the index is out of bounds
                pub fn set(&self, index: usize, value: $ty) {
                    self.data.write().expect("rwlock has been poisoned, cannot get a mutable ref to array element")[index] = value;
                }

                /// Get the length of the array
                pub fn len(&self) -> usize {
                    self.data.read().expect("rwlock has been poisoned, cannot get length to array element").len()
                }
            }

            impl From<Vec<$ty>> for $name {
                fn from(data: Vec<$ty>) -> Self {
                    Self { data: RwLock::new(data) }
                }
            }

            impl From<$name> for Vec<$ty> {
                fn from(array: $name) -> Self {
                    array.data.into_inner().expect("rwlock has been poisoned, cannot consume it to access the array data")
                }
            }
        };
    }
}