
#[derive(Eq, PartialEq, Debug)]
pub enum IdentifyResult {
    Positive,
    Negative,
    NeedMoreData,
}

pub trait TrafficIdentifier {
    fn identify(&self, data: &[u8]) -> IdentifyResult;
}