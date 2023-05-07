use crate::identifier::IdentifyResult;
use tls_parser::parse_tls_plaintext;
use tls_parser::Err::Incomplete;

pub fn identify(data: &[u8]) -> IdentifyResult {
    match parse_tls_plaintext(data) {
        Ok(_) | Err(Incomplete(_)) => IdentifyResult::Positive,
        _ => IdentifyResult::Negative,
    }
}
