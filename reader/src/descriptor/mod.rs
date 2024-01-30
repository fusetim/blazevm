use nom::IResult;
use snafu::Snafu;

pub use self::field::*;
pub use self::class::*;
pub use self::method::*;

pub mod field;
pub mod class;
pub mod method;

#[derive(Debug, Snafu)]
pub enum DescriptorError {
    #[snafu(display("Undecodable descriptor: {}", input))]
    UndecodableDescriptor {
        input: String,
        //source: nom::Err<nom::error::Error<&'static str>>,
    },

    #[snafu(display("Badly formated descriptor as it is longer than the parser decoded it, input:; {}", input))]
    TooLongDescriptor {
        input: String,
    },
}

/// Parse a field descriptor
pub fn parse_field_descriptor(input: &str) -> Result<FieldDescriptor, DescriptorError> {
    let (rem, fty) = field::FieldDescriptor::parse(input).map_err(|err| DescriptorError::UndecodableDescriptor { input: input.into()})?;
    if rem.is_empty() {
        Ok(fty)
    } else {
        Err(DescriptorError::TooLongDescriptor { input: input.into() })
    }
}

/// Parse a method descriptor
pub fn parse_method_descriptor(input: &str) -> Result<MethodDescriptor, DescriptorError> {
    let (rem, mty) = method::MethodDescriptor::parse(input).map_err(|err| DescriptorError::UndecodableDescriptor { input: input.into()})?;
    if rem.is_empty() {
        Ok(mty)
    } else {
        Err(DescriptorError::TooLongDescriptor { input: input.into() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_field_descriptor() {
        assert_eq!(*parse_field_descriptor("B").unwrap().field_type(), FieldType::BaseType(BaseType::Byte));
        assert_eq!(*parse_field_descriptor("C").unwrap().field_type(), FieldType::BaseType(BaseType::Char));
        assert_eq!(*parse_field_descriptor("D").unwrap().field_type(), FieldType::BaseType(BaseType::Double));
        assert_eq!(*parse_field_descriptor("F").unwrap().field_type(), FieldType::BaseType(BaseType::Float));
        assert_eq!(*parse_field_descriptor("I").unwrap().field_type(), FieldType::BaseType(BaseType::Int));
        assert_eq!(*parse_field_descriptor("J").unwrap().field_type(), FieldType::BaseType(BaseType::Long));
        assert_eq!(*parse_field_descriptor("S").unwrap().field_type(), FieldType::BaseType(BaseType::Short));
        assert_eq!(*parse_field_descriptor("Z").unwrap().field_type(), FieldType::BaseType(BaseType::Boolean));
    }

    #[test]
    fn object_field_descriptor() {
        let object = ObjectType::new(ClassName::new(vec![UnqualifiedName::new("java"), UnqualifiedName::new("lang"), UnqualifiedName::new("Object")]));
        let string = ObjectType::new(ClassName::new(vec![UnqualifiedName::new("java"), UnqualifiedName::new("lang"), UnqualifiedName::new("String")]));
        assert_eq!(*parse_field_descriptor("Ljava/lang/Object;").unwrap().field_type(), FieldType::ObjectType(object));
        assert_eq!(*parse_field_descriptor("Ljava/lang/String;").unwrap().field_type(), FieldType::ObjectType(string));
        assert!(parse_field_descriptor("Ljava/lang/Object").is_err());
        assert!(parse_field_descriptor("Ljava/lang/Object;;").is_err());
        assert!(parse_field_descriptor("L[java/lang/Object;").is_err());
    }

    #[test]
    fn array_field_descriptor() {
        let object = ObjectType::new(ClassName::new(vec![UnqualifiedName::new("java"), UnqualifiedName::new("lang"), UnqualifiedName::new("Object")]));
        let string = ObjectType::new(ClassName::new(vec![UnqualifiedName::new("java"), UnqualifiedName::new("lang"), UnqualifiedName::new("String")]));
        assert_eq!(*parse_field_descriptor("[Ljava/lang/Object;").unwrap().field_type(), FieldType::ArrayType(ArrayType::new(FieldType::ObjectType(object))));
        assert_eq!(*parse_field_descriptor("[Ljava/lang/String;").unwrap().field_type(), FieldType::ArrayType(ArrayType::new(FieldType::ObjectType(string))));
        assert_eq!(*parse_field_descriptor("[B").unwrap().field_type(), FieldType::ArrayType(ArrayType::new(FieldType::BaseType(BaseType::Byte))));
        assert!(parse_field_descriptor("[[[B").is_ok());
        assert!(parse_field_descriptor("[[[").is_err());
    }
}