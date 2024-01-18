#[cfg(test)]
#[path = "../src/utils.rs"]
mod test {
    use kivi_rs::utils::{
        build_url, create_path_linter, create_str_linter, decodeb_64_safe, identity_str,
    };

    #[test]
    fn test_identity_str() {
        let input = "a-string";
        assert_eq!(input, identity_str(input));
    }

    #[test]
    fn test_decode_b64_safe_empty_input() {
        let input = "";
        assert_eq!(input, decodeb_64_safe(input));
    }

    #[test]
    fn test_decode_b64_safe() {
        let input = "SGVsbG8sIHdvcmxkIQ==";
        assert_eq!("Hello, world!", decodeb_64_safe(input));
    }

    #[test]
    fn test_decode_b64_safe_with_invalid_input() {
        let input = "SGVsbG8sIHdvcmxkIQ";
        assert_eq!("", decodeb_64_safe(input));
    }
}
