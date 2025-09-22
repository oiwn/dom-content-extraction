use std::fs;

#[test]
fn test_non_utf8_file_fails_with_current_implementation() {
    // This test demonstrates the current limitation:
    // Windows-1251 encoded files fail to be read
    let result = fs::read_to_string("html/test_windows1251.html");

    // This should fail with "stream did not contain valid UTF-8"
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("valid UTF-8"));
}

#[test]
fn test_utf8_file_handling_works() {
    // Test that UTF-8 files work correctly with current implementation
    let result = fs::read_to_string("html/test_1.html");
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}
