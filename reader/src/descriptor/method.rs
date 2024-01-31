use super::field::FieldType;
use nom::{branch::alt, bytes::complete::tag, combinator::map, IResult};

/// Method descriptor representation
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MethodDescriptor {
    pub parameters: Vec<FieldType>,
    pub return_type: Option<FieldType>,
}

impl MethodDescriptor {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, parameters) = parse_parameters(input)?;
        let (input, return_type) = parse_return_type(input)?;
        Ok((
            input,
            Self {
                parameters,
                return_type,
            },
        ))
    }

    pub fn args_count(&self) -> usize {
        self.parameters.len()
    }
}

fn parse_parameters(input: &str) -> IResult<&str, Vec<FieldType>> {
    let (input, _) = tag("(")(input)?;
    let (input, parameters) = nom::multi::many0(FieldType::parse_field_type)(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, parameters))
}

fn parse_return_type(input: &str) -> IResult<&str, Option<FieldType>> {
    let (input, return_type) = alt((
        map(FieldType::parse_field_type, Some),
        map(tag("V"), |_| None),
    ))(input)?;
    Ok((input, return_type))
}
