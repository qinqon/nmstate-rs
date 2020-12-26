#[derive(Debug)]
pub enum ErrorKind {
    InvalidArgument,
    PluginFailure,
    Bug,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for NmstateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.msg)
    }
}

#[derive(Debug)]
pub struct NmstateError {
    kind: ErrorKind,
    msg: String,
}

impl NmstateError {
    pub fn bug(msg: String) -> Self {
        Self {
            kind: ErrorKind::Bug,
            msg: msg,
        }
    }
    pub fn invalid_argument(msg: String) -> Self {
        Self {
            kind: ErrorKind::InvalidArgument,
            msg: msg,
        }
    }
    pub fn plugin_failure(msg: String) -> Self {
        Self {
            kind: ErrorKind::PluginFailure,
            msg: msg,
        }
    }
}
