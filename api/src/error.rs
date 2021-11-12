use std::borrow::Cow;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    /// Misuse errors are caused by misusing, e.g. wrong type of arguments. Always terminate the process.
    Misuse,

    /// Fatal errors suggest that some (shared) states might have been corupted and the whole process should abort.
    Fatal,

    /// NonFatal errors means this stream is failed and should be closed but may not affect other streams.
    NonFatal
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    msg: Cow<'static, str>,
    source: Option<Box<dyn std::error::Error>>
}

impl Error {
    pub fn misuse(msg: impl Into<Cow<'static, str>>, source: Option<Box<dyn std::error::Error>>) -> Error {
        Error { kind: ErrorKind::Misuse, msg: msg.into(), source }
    }

    pub fn fatal(msg: impl Into<Cow<'static, str>>, source: Option<Box<dyn std::error::Error>>) -> Error {
        Error { kind: ErrorKind::Fatal, msg: msg.into(), source }
    }

    pub fn non_fatal(msg: impl Into<Cow<'static, str>>, source: Option<Box<dyn std::error::Error>>) -> Error {
        Error { kind: ErrorKind::NonFatal, msg: msg.into(), source }
    }
}

// TODO: an API helper that impl on all std::error::Error for easy chaining.

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.kind {
            ErrorKind::Misuse => "Misuse: ",
            ErrorKind::Fatal => "Fatal: ",
            ErrorKind::NonFatal => "Error: ",
        };
        write!(f, "{}{}", prefix, self.msg)?;
        if let Some(source) = self.source.as_ref() {
            write!(f, "{}", source)?
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref()
    }
}
