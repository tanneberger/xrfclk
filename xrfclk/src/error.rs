use std::fmt;

#[derive(Debug, Clone)]
pub struct XRFClkError {
    kind: XRFClkErrorKind,
}

#[derive(Debug, Clone)]
pub enum XRFClkErrorKind {
    UnknownError = 0,
    IOError = 1,
    InvalidFrequency = 2,
    InvalidChipString = 3,
    InvalidFilePath = 4,
}

impl fmt::Display for XRFClkErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_string: &str = match self {
            Self::UnknownError => "UnknownError",
            Self::IOError => "IOError",
            Self::InvalidFrequency => "InvalidFrequency",
            Self::InvalidChipString => "InvalidChipString",
            Self::InvalidFilePath => "InvalidFilePath",
        };
        write!(f, "{err_string}")
    }
}

impl fmt::Display for XRFClkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "xrfclk error of kind: {} occurred", self.kind)
    }
}

impl XRFClkError {
    pub fn from(kind: XRFClkErrorKind) -> Self {
        Self { kind }
    }
}

impl From<std::io::Error> for XRFClkError {
    fn from(_: std::io::Error) -> XRFClkError {
        Self::from(XRFClkErrorKind::IOError)
    }
}
