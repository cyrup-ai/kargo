use std::fmt;

#[derive(Debug)]
pub enum BuilderError {
    MissingHandler,
    InvalidRegex(String),
    RegexSetFailed(String),
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuilderError::MissingHandler => write!(f, "PluginBuilder::on_execute not called"),
            BuilderError::InvalidRegex(pattern) => write!(f, "Invalid regex pattern: {}", pattern),
            BuilderError::RegexSetFailed(msg) => write!(f, "Failed to build regex set: {}", msg),
        }
    }
}

impl std::error::Error for BuilderError {}

pub type Result<T> = std::result::Result<T, BuilderError>;
