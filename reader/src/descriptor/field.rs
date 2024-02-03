use super::class::ClassName;
use dumpster::Collectable;
use nom::{branch::alt, bytes::complete::tag, combinator::map, IResult};

/// Field descriptor representation
#[derive(Debug, Clone, Eq, PartialEq, Collectable)]
pub struct FieldDescriptor(FieldType);

impl FieldDescriptor {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, field_type) = FieldType::parse_field_type(input)?;
        Ok((input, Self(field_type)))
    }

    pub fn field_type(&self) -> &FieldType {
        &self.0
    }

    /// Get the class name of the referenced class
    ///
    /// If the field type is an object type, return the class name of the object type.
    /// If is an (multi-)array type, return the referenced class name of the item type if it exists.
    pub fn get_referenced_class(&self) -> Option<&ClassName> {
        let mut field_type = &self.0;
        loop {
            match field_type {
                FieldType::BaseType(_) => return None,
                FieldType::ObjectType(object_type) => return Some(&object_type.class_name),
                FieldType::ArrayType(array_type) => {
                    field_type = array_type.item.as_ref();
                }
            }
        }
    }
}

/// Field type representation
///
/// Dispatch to one of the 3 types of types: primitive, object or array.
#[derive(Debug, Clone, Eq, PartialEq, Collectable)]
pub enum FieldType {
    BaseType(BaseType),
    ObjectType(ObjectType),
    ArrayType(ArrayType),
}

impl FieldType {
    pub fn parse_field_type(input: &str) -> IResult<&str, Self> {
        alt((
            map(BaseType::parse, Self::BaseType),
            map(ObjectType::parse, Self::ObjectType),
            map(ArrayType::parse, Self::ArrayType),
        ))(input)
    }

    pub fn is_primitive(&self) -> bool {
        match self {
            Self::BaseType(_) => true,
            _ => false,
        }
    }

    pub fn is_object(&self) -> bool {
        match self {
            Self::ObjectType(_) => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            Self::ArrayType(_) => true,
            _ => false,
        }
    }
}

/// Primitive type representation
#[derive(Debug, Clone, Eq, PartialEq, Collectable)]
pub enum BaseType {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
}

impl BaseType {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            map(tag("B"), |_| Self::Byte),
            map(tag("C"), |_| Self::Char),
            map(tag("D"), |_| Self::Double),
            map(tag("F"), |_| Self::Float),
            map(tag("I"), |_| Self::Int),
            map(tag("J"), |_| Self::Long),
            map(tag("S"), |_| Self::Short),
            map(tag("Z"), |_| Self::Boolean),
        ))(input)
    }
}

/// Object type representation
///
/// An object type is represented mostly by its class name.
#[derive(Debug, Clone, Eq, PartialEq, Collectable)]
pub struct ObjectType {
    pub class_name: ClassName,
}

impl ObjectType {
    pub fn new(class_name: ClassName) -> Self {
        Self { class_name }
    }

    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("L")(input)?;
        let (input, class_name) = ClassName::parse(input)?;
        let (input, _) = tag(";")(input)?;
        Ok((input, Self { class_name }))
    }
}

/// Array type representation
#[derive(Debug, Clone, Eq, PartialEq, Collectable)]
pub struct ArrayType {
    pub item: Box<FieldType>,
}

impl ArrayType {
    pub fn new(item: FieldType) -> Self {
        Self {
            item: Box::new(item),
        }
    }

    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("[")(input)?;
        let (input, item) = FieldType::parse_field_type(input)?;
        Ok((
            input,
            Self {
                item: Box::new(item),
            },
        ))
    }

    pub fn item(&self) -> &FieldType {
        &self.item
    }
}
