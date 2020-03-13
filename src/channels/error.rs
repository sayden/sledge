#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("'{0}' parsing error")]
    ParsingError(String),

    #[error("target field with key '{0}' not found in incoming json")]
    FieldNotFoundInJSON(String),

    #[error("key '{0}' not found in mutator")]
    KeyNotFoundInMutator(String),

    #[error("value '{0}' in JSON is not a string")]
    NotString(String),

    #[error("value of key '{0}' is not an array")]
    NotAnArray(String),

    #[error("value of key '{0}' is not an i64")]
    NotI64(String),

    #[error("value of key '{0}' is not a bool")]
    NotBool(String),

    #[error("value of key '{0}' is not an array")]
    ValueInArrayIsNotString(String),

    #[error("required field '{0}' not found in mutator")]
    RequiredFieldNotFound(String),

    #[error("grok error: '{0}'")]
    GrokError(#[from]grok::Error),

    #[error("grok found no matches")]
    GrokNoMatches,

    #[error("array '{0}' is empty")]
    EmptyArray(String),

    #[error("only strings and numbers can be sorted")]
    SortNotPossibleError,

    #[error("empty 'separator' field in 'split' mutator")]
    SplitEmptySeparator,

    #[error("mutator not found '{0}'")]
    MutatorNotFound(String),

    #[error("only 'string' and 'array' are allowed values in the uppper/lower case mutator")]
    UpperLowerCaseErrorTypeNotRecognized,

    #[error("type not recognized in 'join' mutator. Only string and array are allowed")]
    JoinErrorTypeNotRecognized,

    #[error("cannot remove field '{0}'")]
    CannotRemoveField(String),
}

impl From<Error> for Result<(), Error> {
    fn from(e: Error) -> Self {
        Err(e)
    }
}