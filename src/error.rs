use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidHeader,
    InvalidVersion,
    InvalidOemCodePage,
    UnexpectedMessageAttribute,
    ChecksumMismatch,
    InvalidAttributeLevel(u8),
    InvalidMessageId(u32),
    InvalidAttachAttr(u32),
    UnexpectedEof,
    InvlidDateTime,
    InvalidRendData,
    InvalidString,
    AttachmentParsingFailure,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error { }
