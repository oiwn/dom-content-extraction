use chardetng::EncodingDetector;
use encoding_rs::Encoding;
use std::fs;

/// Same decoding strategy as `examples/ce_score.rs`:
/// fast-path UTF-8, otherwise detect encoding and decode.
fn decode_bytes(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    let mut detector = EncodingDetector::new(chardetng::Iso2022JpDetection::Deny);
    detector.feed(bytes, true);
    let encoding: &Encoding = detector.guess(None, chardetng::Utf8Detection::Allow);
    let (decoded, _, _had_errors) = encoding.decode(bytes);
    decoded.into_owned()
}

#[test]
fn test_non_utf8_file_is_detected_and_decoded() {
    // Windows-1251 files must no longer fail with
    // "stream did not contain valid UTF-8" (issue: CleanEval file 730)
    let bytes = fs::read("html/test_windows1251.html").expect("fixture must exist");

    // Sanity check: raw bytes are indeed not valid UTF-8
    assert!(std::str::from_utf8(&bytes).is_err());

    let content = decode_bytes(&bytes);
    assert!(content.contains("Привет мир"));
    assert!(content.contains("Это тест на русском языке"));
}

#[test]
fn test_utf8_file_handling_works() {
    // Test that UTF-8 files work correctly with current implementation
    let result = fs::read_to_string("html/test_1.html");
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}
