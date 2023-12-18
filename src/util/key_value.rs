/// Auxiliary struct for parsing key-value pairs.
///
/// # Example
///
/// ```
/// # use rosu_map::util::KeyValue;
/// let KeyValue { key, value } = KeyValue::new("key: value");
/// assert_eq!(key, "key");
/// assert_eq!(value, "value");
///
/// let KeyValue { key, value } = KeyValue::new("key   :  ");
/// assert_eq!(key, "key");
/// assert_eq!(value, "");
///
/// let KeyValue { key, value } = KeyValue::new(" key   ");
/// assert_eq!(key, "key");
/// assert_eq!(value, "");
/// ```
pub struct KeyValue<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

impl<'a> KeyValue<'a> {
    /// Create a new [`KeyValue`] pair by splitting on the first `:`.
    pub fn new(s: &'a str) -> Self {
        let mut split = s.split(':').map(str::trim);

        Self {
            key: split.next().unwrap_or(s.trim()),
            value: split.next().unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_and_value() {
        let kv = KeyValue::new("key:value");
        assert_eq!(kv.key, "key");
        assert_eq!(kv.value, "value");

        let kv = KeyValue::new("  key    :  value   ");
        assert_eq!(kv.key, "key");
        assert_eq!(kv.value, "value");
    }

    #[test]
    fn only_key() {
        let kv = KeyValue::new("key:");
        assert_eq!(kv.key, "key");
        assert_eq!(kv.value, "");

        let kv = KeyValue::new("   key  :   ");
        assert_eq!(kv.key, "key");
        assert_eq!(kv.value, "");
    }

    #[test]
    fn only_value() {
        let kv = KeyValue::new(":value");
        assert_eq!(kv.key, "");
        assert_eq!(kv.value, "value");

        let kv = KeyValue::new("  :  value     ");
        assert_eq!(kv.key, "");
        assert_eq!(kv.value, "value");
    }

    #[test]
    fn no_colon() {
        let kv = KeyValue::new("key value");
        assert_eq!(kv.key, "key value");
        assert_eq!(kv.value, "");
    }
}
