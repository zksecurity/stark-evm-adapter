use ethers::{
    abi::{
        ethereum_types::{FromDecStrErr, FromStrRadixErr},
        EncodePackedError,
    },
    utils::hex::FromHexError,
};
use num_bigint::ParseBigIntError;
use std::num::ParseIntError;
use thiserror::Error;

// Define the custom error type using `thiserror`.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("regex error")]
    RegexError,
    #[error("parse u256 error")]
    ParseU256Error,
    #[error("parse integer error")]
    ParseIntError,
    #[error("parse bigint error")]
    ParseBigUIntError,
    #[error("hex extraction error")]
    HexExtractionError,
    #[error("invalid line format")]
    InvalidLineFormat,
    #[error("ether encoding error")]
    EtherEncodingError,
}

impl From<FromDecStrErr> for ParseError {
    fn from(_: FromDecStrErr) -> Self {
        ParseError::ParseU256Error
    }
}

impl From<FromStrRadixErr> for ParseError {
    fn from(_: FromStrRadixErr) -> Self {
        ParseError::ParseU256Error
    }
}

impl From<ParseBigIntError> for ParseError {
    fn from(_: ParseBigIntError) -> Self {
        ParseError::ParseBigUIntError
    }
}

impl From<regex::Error> for ParseError {
    fn from(_: regex::Error) -> Self {
        ParseError::RegexError
    }
}

impl From<ParseIntError> for ParseError {
    fn from(_: ParseIntError) -> Self {
        ParseError::RegexError
    }
}

impl From<FromHexError> for ParseError {
    fn from(_: FromHexError) -> Self {
        ParseError::InvalidLineFormat
    }
}

impl From<EncodePackedError> for ParseError {
    fn from(_: EncodePackedError) -> Self {
        ParseError::EtherEncodingError
    }
}
