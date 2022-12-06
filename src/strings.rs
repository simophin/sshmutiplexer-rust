use crate::identifier::IdentifyResult;

pub fn raw_string_matches(
    test: &[u8],
    starts_with_needle: &[u8],
) -> IdentifyResult {
    if test.iter().zip(starts_with_needle.iter())
        .filter(|(a, b)| *a != *b)
        .next()
        .is_some() {
        return IdentifyResult::Negative
    }

    if test.len() >= starts_with_needle.len() {
        IdentifyResult::Positive
    } else {
        IdentifyResult::NeedMoreData
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_test_negative() {
        assert_eq!(
            raw_string_matches(
                b"HTTP GET /",
                b"SSH-",
            ),
            IdentifyResult::Negative
        );
    }

    #[test]
    fn should_test_positive() {
        assert_eq!(
            raw_string_matches(
                b"HTTP GET /",
                b"HTTP ",
            ),
            IdentifyResult::Positive
        );
    }

    #[test]
    fn should_need_more_data() {
        assert_eq!(
            raw_string_matches(
                b"HTT",
                b"HTTP ",
            ),
            IdentifyResult::NeedMoreData
        );
    }
}