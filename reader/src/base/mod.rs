mod classfile;
mod constant_pool;

pub use constant_pool::{ConstantPool};
pub use classfile::ClassFile;

pub type U1 = u8;
pub type U2 = u16;
pub type U4 = u32;