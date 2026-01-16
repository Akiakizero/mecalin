pub fn language_from_locale() -> &'static str {
    let locale = std::env::var("LANG").unwrap_or_else(|_| "en_US".to_string());
    if locale.starts_with("es") {
        "es"
    } else if locale.starts_with("it") {
        "it"
    } else {
        "us"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_locale_spanish() {
        std::env::set_var("LANG", "es_ES.UTF-8");
        assert_eq!(language_from_locale(), "es");
    }

    #[test]
    fn test_language_from_locale_italian() {
        std::env::set_var("LANG", "it_IT.UTF-8");
        assert_eq!(language_from_locale(), "it");
    }

    #[test]
    fn test_language_from_locale_english() {
        std::env::set_var("LANG", "en_US.UTF-8");
        assert_eq!(language_from_locale(), "us");
    }

    #[test]
    fn test_language_from_locale_default() {
        std::env::set_var("LANG", "xx_YY.UTF-8");
        assert_eq!(language_from_locale(), "us");
    }

    #[test]
    fn test_language_from_locale_partial_match() {
        std::env::set_var("LANG", "es");
        assert_eq!(language_from_locale(), "es");
    }
}
