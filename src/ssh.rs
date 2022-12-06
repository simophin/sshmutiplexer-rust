use crate::strings::raw_string_matches;
use super::identifier::{IdentifyResult, TrafficIdentifier};

pub struct SSHIdentifier;

impl TrafficIdentifier for SSHIdentifier {
    fn identify(&self, data: &[u8]) -> IdentifyResult {
        raw_string_matches(data, b"SSH-")
    }
}