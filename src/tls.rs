use tls_parser::Err::Incomplete;
use tls_parser::parse_tls_plaintext;
use crate::identifier::{IdentifyResult, TrafficIdentifier};

pub struct TLSIdentifier;

impl TrafficIdentifier for TLSIdentifier {
    fn identify(&self, data: &[u8]) -> IdentifyResult {
        match parse_tls_plaintext(data) {
            Ok(_) | Err(Incomplete(_)) => IdentifyResult::Positive,
            _ => IdentifyResult::Negative,
        }
    }
}