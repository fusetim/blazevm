use nom::{branch::alt, bytes::complete::tag, character::complete::none_of, combinator::map, multi::many1, IResult};

/// Classname representation
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClassName {
    parts: Vec<UnqualifiedName>,
}

impl ClassName {
    pub fn new(parts: Vec<UnqualifiedName>) -> Self {
        Self { parts }
    }

    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, parts) = nom::multi::separated_list1(tag("/"), UnqualifiedName::parse)(input)?;
        Ok((input, Self { parts }))
    }

    pub fn as_binary_name(&self) -> String {
        self.parts.iter().map(|part| part.as_str()).collect::<Vec<_>>().join("/")
    }

    pub fn as_source_name(&self) -> String {
        self.parts.iter().map(|part| part.as_str()).collect::<Vec<_>>().join(".")
    }
}

/// Unqualified name representation
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnqualifiedName(pub String);

impl UnqualifiedName {
    pub fn new(name: &str) -> Self {
        Self(name.into())
    }
    
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, name) = many1(none_of("./[;"))(input)?;
        Ok((input, Self(name.into_iter().collect())))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}