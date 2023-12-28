use std::str::FromStr;

/// Auxiliary struct for parsing key-value pairs.
pub struct KeyValue<'a, K> {
    /// Should be of any type that implements [`FromStr`].
    pub key: K,
    /// The whitespace-trimmed content after the colon (`:`).
    pub value: &'a str,
}

impl<'a, K: FromStr> KeyValue<'a, K> {
    /// Create a new [`KeyValue`] pair by splitting on the first `:`
    /// and parsing the key.
    ///
    /// # Example
    ///
    /// ```
    /// use rosu_map::util::KeyValue;
    /// use rosu_map::section::difficulty::DifficultyKey;
    ///
    /// let line = "ApproachRate: 9.3 // Some comment";
    ///
    /// let kv = KeyValue::<DifficultyKey>::parse(line).unwrap();
    ///
    /// assert_eq!(kv.key, DifficultyKey::ApproachRate);
    /// assert_eq!(kv.value, "9.3 // Some comment");
    /// ```
    pub fn parse(s: &'a str) -> Result<Self, K::Err> {
        let mut split = s.split(':').map(str::trim);

        Ok(Self {
            key: split.next().unwrap_or(s.trim()).parse()?,
            value: split.next().unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Key;

    impl FromStr for Key {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "key" => Ok(Self),
                _ => Err(()),
            }
        }
    }

    #[test]
    fn key_and_value() {
        let kv = KeyValue::<Key>::parse("key:value").unwrap();
        assert_eq!(kv.key, Key);
        assert_eq!(kv.value, "value");

        let kv = KeyValue::<Key>::parse("  key    :  value   ").unwrap();
        assert_eq!(kv.key, Key);
        assert_eq!(kv.value, "value");
    }

    #[test]
    fn only_key() {
        let kv = KeyValue::<Key>::parse("key:").unwrap();
        assert_eq!(kv.key, Key);
        assert_eq!(kv.value, "");

        let kv = KeyValue::<Key>::parse("   key  :   ").unwrap();
        assert_eq!(kv.key, Key);
        assert_eq!(kv.value, "");
    }

    #[test]
    fn only_value() {
        assert!(KeyValue::<Key>::parse(":value").is_err());
        assert!(KeyValue::<Key>::parse("  :  value     ").is_err());
    }

    #[test]
    fn no_colon() {
        assert!(KeyValue::<Key>::parse("key value").is_err());
    }
}
