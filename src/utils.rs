pub fn language_from_locale() -> &'static str {
    let locale = std::env::var("LANG").unwrap_or_else(|_| "en_US".to_string());
    if locale.starts_with("es") {
        "es"
    } else if locale.starts_with("it") {
        "it"
    } else if locale.starts_with("pl") {
        "pl"
    } else {
        "us"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_language_from_locale_spanish() {
        let _lock = TEST_MUTEX.lock().unwrap();
        std::env::set_var("LANG", "es_ES.UTF-8");
        assert_eq!(language_from_locale(), "es");
    }

    #[test]
    fn test_language_from_locale_italian() {
        let _lock = TEST_MUTEX.lock().unwrap();
        std::env::set_var("LANG", "it_IT.UTF-8");
        assert_eq!(language_from_locale(), "it");
    }

    #[test]
    fn test_language_from_locale_english() {
        let _lock = TEST_MUTEX.lock().unwrap();
        std::env::set_var("LANG", "en_US.UTF-8");
        assert_eq!(language_from_locale(), "us");
    }

    #[test]
    fn test_language_from_locale_default() {
        let _lock = TEST_MUTEX.lock().unwrap();
        std::env::set_var("LANG", "xx_YY.UTF-8");
        assert_eq!(language_from_locale(), "us");
    }

    #[test]
    fn test_language_from_locale_polish() {
        let _lock = TEST_MUTEX.lock().unwrap();
        std::env::set_var("LANG", "pl_PL.UTF-8");
        assert_eq!(language_from_locale(), "pl");
    }

    #[test]
    fn test_language_from_locale_partial_match() {
        let _lock = TEST_MUTEX.lock().unwrap();
        std::env::set_var("LANG", "es");
        assert_eq!(language_from_locale(), "es");
    }
}
