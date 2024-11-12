use derive_more::derive::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Custom(String),

    NoPlayerFound,
    InvalidTarget,

    // -- Internals
    #[from]
    MonsterTurnError(crate::system::error::MonsterTurnError),

    //-- Externals
    #[from]
    SerdeYaml(serde_yaml::Error),
    #[from]
    Io(std::io::Error),
}

impl Error {
    fn custom(value: impl std::fmt::Display) -> Self {
        Self::Custom(value.to_string())
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Custom(value.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
