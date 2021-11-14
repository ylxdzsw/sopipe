use serde::Deserialize;

/// An enum type that represents user arguments
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    String(String),
    Int(u64),
    Vec(Vec<Argument>),
    None
}

impl From<String> for Argument {
    fn from(x: String) -> Self {
        Self::String(x)
    }
}

impl From<u64> for Argument {
    fn from(x: u64) -> Self {
        Self::Int(x)
    }
}

impl<T> FromIterator<T> for Argument where Argument: std::convert::From<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        Self::Vec(iter.into_iter().map(Argument::from).collect())
    }
}

impl Argument {
    pub fn type_name(&self) -> &'static str {
        match &self {
            Argument::String(_) => "string",
            Argument::Int(_) => "int",
            Argument::Vec(_) => "vec",
            Argument::None => "none",
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match &self {
            &Argument::String(x) => Some(x),
            _ => None
        }
    }

    pub fn as_int(&self) -> Option<&u64> {
        match &self {
            &Argument::Int(x) => Some(x),
            _ => None
        }
    }

    pub fn as_vec(&self) -> Option<&[Argument]> {
        match &self {
            &Argument::Vec(x) => Some(x),
            _ => None
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, &Argument::None)
    }
}

// for #[serde(default)]
impl Default for Argument {
    fn default() -> Self {
        Argument::None
    }
}
