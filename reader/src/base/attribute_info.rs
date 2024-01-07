use super::{ConstantPool, U1, U2, U4, StackMapFrame, stack_frame::parse_stack_map_frame};
use binrw::{binrw, BinRead, BinReaderExt, BinResult};
use flagset::{flags, FlagSet};

#[derive(BinRead)]
#[br(big)]
pub struct AttributeInfo {
    /// Unqualified name denoting the attribute.
    /// The index must point to a valid [crate::base::constant_pool::Utf8Info] in the constant pool.
    pub attribute_name_index: U2,
    // Info length
    pub attribute_length: U4,
    /// Variable-length info
    #[br(count=attribute_length)]
    pub info: Vec<U1>,
}

/// Attribute ConstantValue, a member of [AttributeInfo].
///
/// Represents the value (by reference) of a constant field.
#[derive(BinRead)]
#[br(big)]
pub struct ConstantValueAttribute {
    /// Index of the constant in the constant pool.
    /// The constant must be of the same type as the field.
    constant_value_index: U2,
}

/// Attribute Code, a member of [AttributeInfo].
///
/// Represents the body of a method.
/// It contains the bytecode, the exception table, and the attributes,
/// and some auxiliary information.
#[derive(BinRead)]
#[br(big)]
pub struct CodeAttribute {
    /// The max length of the operand stack of this method.
    max_stack: U2,
    /// The maximum number of local variables in the local variable array allocated
    /// upon invocation of this method.
    max_locals: U2,
    /// The number of bytes in the bytecode array.
    code_length: U4,
    /// The bytecode array.
    #[br(count=code_length)]
    code: Vec<U1>,
    /// The number of entries in the exception table.
    exception_table_length: U2,
    /// The exception table.
    #[br(count=exception_table_length)]
    exception_table: Vec<ExceptionTableEntry>,
    /// The number of attributes in the attributes table.
    attributes_count: U2,
    /// The attributes table.
    #[br(count=attributes_count)]
    attributes: Vec<AttributeInfo>,
}

/// Entry of the exception table of a [CodeAttribute].
#[derive(BinRead)]
#[br(big)]
pub struct ExceptionTableEntry {
    /// Indicates the start of the code range where the exception handler is active.
    start_pc: U2,
    /// Indicates the end of the code range where the exception handler is active.
    end_pc: U2,
    /// Indicates the first instruction of the exception handler to run.
    handler_pc: U2,
    /// Index of a [ClassInfo] in the constant pool.
    ///
    /// If non-zero, it represents the Exception class of exception handled by the catch clause.
    /// If zero, it represents a catch clause that handles all types of exceptions.
    catch_type: U2,
}

/// Atribute StackMapTable, a member of [AttributeInfo].
///
/// Represents the stack map table of a method.
#[derive(BinRead)]
#[br(big)]
pub struct StackMapTableAttribute {
    /// The number of entries in the stack map table.
    number_of_entries: U2,
    /// The stack map table.
    #[br(parse_with=parse_stack_map_entries, args(number_of_entries as usize))]
    entries: Vec<StackMapFrame>,
}

#[binrw::parser(reader, endian)]
fn parse_stack_map_entries(count: usize) -> BinResult<Vec<StackMapFrame>> {
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let entry: StackMapFrame = parse_stack_map_frame(reader, endian, ())?;
        entries.push(entry);
    }
    Ok(entries)
}

/// Attribute BootstrapMethods, a member of [AttributeInfo].
///
/// This attribute records bootstrap methods used by dynamic instructions.
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.23>
#[derive(BinRead)]
#[br(big)]
pub struct BootstrapMethodsAttribute {
    /// The number of bootstrap methods in the bootstrap_methods array.
    pub num_bootstrap_methods: U2,

    /// The bootstrap methods.
    #[br(count=num_bootstrap_methods)]
    pub bootstrap_methods: Vec<BootstrapMethod>,
}

/// A bootstrap method, a member of [BootstrapMethodsAttribute].
///
/// This structure represents a bootstrap method, which is a method that is invoked
/// during the invocation of a dynamic instruction.
/// It invokes a method to compute the value of a number of static arguments.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.23>
#[derive(BinRead)]
#[br(big)]
pub struct BootstrapMethod {
    /// A reference to a [MethodHandleInfo] in the constant pool.
    pub bootstrap_method_ref: U2,
    /// The number of items in the bootstrap_arguments array.
    pub num_bootstrap_arguments: U2,
    /// The bootstrap **static** arguments, referenced by their indices in the constant pool.
    #[br(count=num_bootstrap_arguments)]
    pub bootstrap_arguments: Vec<U2>,
}

/// Attribute NestHost, a member of [AttributeInfo].
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.28>
#[derive(BinRead)]
#[br(big)]
pub struct NestHostAttribute {
    /// A reference to a [ClassInfo] in the constant pool.
    ///
    /// The class/interface is the nest host of the current class/interface.
    pub host_class_index: U2,
}

/// Attribute NestMembers, a member of [AttributeInfo].
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.29>
#[derive(BinRead)]
#[br(big)]
pub struct NestMembersAttribute {
    /// The number of entries in the classes array.
    pub num_classes: U2,
    /// The classes/interfaces that are members of the nest to which the current class/interface belongs.
    /// Each entry is a reference to a [ClassInfo] in the constant pool.
    #[br(count=num_classes)]
    pub classes: Vec<U2>,
}

/// Attribute PermittedSubclasses, a member of [AttributeInfo].
///
/// This attribute records the classes that are permitted to extend the current class.
///
/// Note: For final classes (cf [ClassAccessFlags::FINAL](super::classfile::ClassAccessFlags)), this 
/// attribute MUST exist and MUST be empty.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.31>
#[derive(BinRead)]
#[br(big)]
pub struct PermittedSubclassesAttribute {

    pub num_classes: U2,
    #[br(count=num_classes)]
    pub classes: Vec<U2>,
}

/// Attribute Exceptions, a member of [AttributeInfo].
///
/// This attribute records the exceptions that a method is declared to throw.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.5>
#[derive(BinRead)]
#[br(big)]
pub struct ExceptionsAttribute {
    /// The number of entries in the exception_index_table.
    pub number_of_exceptions: U2,
    /// The list of exceptions that the method is declared to throw.
    /// Each entry is a reference to a [ClassInfo](super::constant_pool::ClassInfo) in the constant pool.
    #[br(count=number_of_exceptions)]
    pub exception_index_table: Vec<U2>,
}


/// Attribute InnerClasses, a member of [AttributeInfo].
///
/// This attribute records the inner classes of a class or interface.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.6>
pub struct InnerClassesAttribute {
    /// The number of entries in the classes array.
    pub number_of_classes: U2,
    /// References all the class/interface that are represented as a [ClassInfo](super::constant_pool::ClassInfo) 
    /// in the constant pool, but that are not a member of a package.
    pub classes: Vec<InnerClass>,
}

/// An inner class, a structure part of [InnerClassesAttribute].
#[derive(BinRead)]
#[br(big)]
pub struct InnerClass {
    /// A reference to a [ClassInfo](super::constant_pool::ClassInfo) in the constant pool.
    ///
    /// The class or interface, the inner class, that is a member of the current class or interface.
    pub inner_class_info_index: U2,
    /// A reference to a [ClassInfo](super::constant_pool::ClassInfo) in the constant pool.
    ///
    /// The class or interface of which the current class or interface is a member.
    /// If the current class or interface is not a member of a class or interface, the value of the
    /// outer_class_info_index item must be zero.
    pub outer_class_info_index: U2,
    /// A reference to a [Utf8Info](super::constant_pool::Utf8Info) in the constant pool.
    ///
    /// The simple name of the current class or interface.
    /// If the current class or interface is anonymous, the value of the inner_name_index item must be zero.
    pub inner_name_index: U2,
    /// The access flags of the current class or interface as a member of the class or interface
    /// specified by the outer_class_info_index.
    #[br(map= |x: U2| FlagSet::<InnerClassAccessFlags>::new_truncated(x))]
    pub inner_class_access_flags: FlagSet<InnerClassAccessFlags>,
}

flags! {
    /// Inner Access flags for classes, interfaces and modules.
    /// See <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.6-300-D.1-D.1>.
    pub enum InnerClassAccessFlags: U2 {
        /// Declared public; may be accessed from outside its package.
        Public = 0x0001,
        /// Declared private; usable only within the defining class.
        Private = 0x0002,
        /// Declared protected; may be accessed within subclasses.
        Protected = 0x0004,
        /// Declared static.
        Static = 0x0008,
        /// Declared final; no subclasses allowed.
        Final = 0x0010,
        /// Is an interface, not a class.
        Interface = 0x0200,
        /// Declared abstract; must not be instantiated.
        Abstract = 0x0400,
        /// Declared synthetic; not present in the source code.
        Synthetic = 0x1000,
        /// Declared as an annotation interface.
        Annotation = 0x2000,
        /// Declared as an enum class.
        Enum = 0x4000,
    }
}

/// Attribute EnclosingMethod, a member of [AttributeInfo].
///
/// This attribute records the enclosing method of a class.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.7>
#[derive(BinRead)]
#[br(big)]
pub struct EnclosingMethodAttribute {
    /// A reference to a [ClassInfo](super::constant_pool::ClassInfo) in the constant pool.
    ///
    /// The class that encloses the current class.
    pub class_index: U2,
    /// A reference to a [NameAndTypeInfo](super::constant_pool::NameAndTypeInfo) in the constant pool.
    ///
    /// The name and type of a method in the class referenced by the class_index.
    /// The referenced method is the enclosing method of the current class.
    /// If the current class is not immediately enclosed by a method, then the value of the
    /// method_index item must be zero.
    pub method_index: U2,
}

/// Attribute Synthetic, a member of [AttributeInfo].
///
/// This attribute is a marker attribute, with no information.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.8>
pub struct SyntheticAttribute {}

/// Attribute Signature, a member of [AttributeInfo].
///
/// This attribute records the signature of a class, field, or method.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.9>
#[derive(BinRead)]
#[br(big)]
pub struct SignatureAttribute {
    /// A reference to a [Utf8Info](super::constant_pool::Utf8Info) in the constant pool.
    ///
    /// The signature of the class, field, or method represented by this [AttributeInfo].
    pub signature_index: U2,
}

/// Attribute Record, a member of [AttributeInfo].
///
/// This attribute records the name and the components of a record.
///
/// Added in Java SE 16.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.30>
#[derive(BinRead)]
#[br(big)]
pub struct RecordAttribute {
    /// The number of entries in the components array.
    pub component_count: U2,
    /// The components of the record.
    #[br(count=component_count)]
    pub components: Vec<RecordComponent>,
}

/// A record component, structure part of [RecordAttribute].
///
/// Added in Java SE 16.
#[derive(BinRead)]
#[br(big)]
pub struct RecordComponent {
    /// A reference to a [Utf8Info](super::constant_pool::Utf8Info) in the constant pool.
    ///
    /// The name of the record component.
    pub name_index: U2,
    /// A reference to a [Utf8Info](super::constant_pool::Utf8Info) in the constant pool.
    ///
    /// The descriptor of the record component.
    pub descriptor_index: U2,
    /// The number of entries in the attributes array.
    pub attributes_count: U2,
    /// The attributes of the record component.
    #[br(count=attributes_count)]
    pub attributes: Vec<AttributeInfo>,
}

/// Attribute SourceFile, a member of [AttributeInfo].
///
/// This attribute records the name of the source file from which the class file was compiled.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.10>
#[derive(BinRead)]
#[br(big)]
pub struct SourceFileAttribute {
    /// A reference to a [Utf8Info] in the constant pool.
    ///
    /// The name of the source file from which this class file was compiled.
    pub sourcefile_index: U2,
}

/// Attribute LineNumberTable, a member of [AttributeInfo].
///
/// This attribute records information about the correspondence between
/// the line numbers of the source code and the indices into the bytecode array.
/// It may be used by debuggers to determine which part of the source code
/// corresponds to a given section of the bytecode array.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.11>
#[derive(BinRead)]
#[br(big)]
pub struct LineNumberTableAttribute {
    /// The number of entries in the line_number_table.
    pub line_number_table_length: U2,
    // The line number table.
    #[br(count=line_number_table_length)]
    pub line_number_table: Vec<LineNumberTableEntry>,
}

/// Entry of the line number table of a [LineNumberTableAttribute].
#[derive(BinRead)]
#[br(big)]
pub struct LineNumberTableEntry {
    /// The index into the bytecode array at which the code for a new line in the original source file begins.
    pub start_pc: U2,
    /// The corresponding line number in the original source file.
    pub line_number: U2,
}

/// Attribute LocalVariableTable, a member of [AttributeInfo].
///
/// This attribute records information about the local variables in the code of a method.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.13>
#[derive(BinRead)]
#[br(big)]
pub struct LocalVariableTableAttribute {
    /// The number of entries in the local_variable_table.
    pub local_variable_table_length: U2,
    /// The local variable table.
    #[br(count=local_variable_table_length)]
    pub local_variable_table: Vec<LocalVariableTableEntry>,
}

/// Entry of the local variable table of a [LocalVariableTableAttribute].
#[derive(BinRead)]
#[br(big)]
pub struct LocalVariableTableEntry {
    /// First instruction corresponding to the start of the scope of the local variable.
    pub start_pc: U2,
    /// The length of the scope of the local variable.
    /// The bytecode at start_pc + length must be the first bytecode at which the local variable is no longer in scope.
    /// This is not necessarily a valid instruction.
    pub length: U2,
    /// A reference to a [Utf8Info](super::constant_pool::Utf8Info) in the constant pool.
    ///
    /// The name of a local variable in the original source file.
    pub name_index: U2,
    /// A reference to a [Utf8Info](super::constant_pool::Utf8Info) in the constant pool.
    ///
    /// A descriptor of a local variable in the original source file.
    pub descriptor_index: U2,
    /// The index of the local variable in the local variable array of the current frame.
    pub index: U2,
}

/// Attribute LocalVariableTypeTable, a member of [AttributeInfo].
///
/// This attribute records information about the types of the local variables in the code of a method.
///
/// Ref: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.14>
#[derive(BinRead)]
#[br(big)]
pub struct LocalVariableTypeTableAttribute {
    /// The number of entries in the local_variable_type_table.
    pub local_variable_type_table_length: U2,
    // The local variable type table.
    #[br(count=local_variable_type_table_length)]
    pub local_variable_type_table: Vec<LocalVariableTypeTableEntry>,
}

/// Entry of the local variable type table of a [LocalVariableTypeTableAttribute].
#[derive(BinRead)]
#[br(big)]
pub struct LocalVariableTypeTableEntry {
    /// First instruction corresponding to the start of the scope of the local variable.
    pub start_pc: U2,
    /// The length of the scope of the local variable.
    /// The bytecode at start_pc + length must be the first bytecode at which the local variable is no longer in scope.
    /// This is not necessarily a valid instruction.
    pub length: U2,
    /// A reference to a [Utf8Info](super::constant_pool::Utf8Info) in the constant pool.
    ///
    /// The name of a local variable in the original source file.
    pub name_index: U2,
    /// A reference to a [Utf8Info](super::constant_pool::Utf8Info) in the constant pool.
    ///
    /// A descriptor of a local variable in the original source file.
    pub signature_index: U2,
    /// The index of the local variable in the local variable array of the current frame.
    pub index: U2,
}