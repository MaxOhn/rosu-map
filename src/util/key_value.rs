pub struct KeyValue<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

impl<'a> KeyValue<'a> {
    pub fn new(s: &'a str) -> Self {
        let mut split = s.split(':').map(str::trim);

        Self {
            key: split.next().unwrap_or(s.trim()),
            value: split.next().unwrap_or_default(),
        }
    }
}
