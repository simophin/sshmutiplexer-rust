use super::identifier::IdentifyResult;
use crate::strings::raw_string_matches;

pub fn identify(data: &[u8]) -> IdentifyResult {
    raw_string_matches(data, b"SSH-")
}
