use crate::util::{ParseNumber, ParseNumberError};

/// Extension methods for `str` which are commonly used in and around this library.
pub trait StrExt {
    /// Trim trailing comments and whitespace.
    fn trim_comment(&self) -> &str;

    /// Parse `&str` to a number without exceeding [`MAX_PARSE_VALUE`].
    ///
    /// [`MAX_PARSE_VALUE`]: crate::util::MAX_PARSE_VALUE
    fn parse_num<N: ParseNumber>(&self) -> Result<N, ParseNumberError>;

    /// Parse `&str` to a number without exceeding the given limit.
    fn parse_with_limits<N: ParseNumber>(&self, limit: N) -> Result<N, ParseNumberError>;

    /// Replace windows path separators with unix ones.
    fn to_standardized_path(&self) -> String;

    /// Fix path and quotation segments to normalize filenames.
    fn clean_filename(&self) -> String;
}

impl StrExt for str {
    fn trim_comment(&self) -> &str {
        self.find("//").map_or(self, |i| self[..i].trim_end())
    }

    fn parse_num<N: ParseNumber>(&self) -> Result<N, ParseNumberError> {
        N::parse(self)
    }

    fn parse_with_limits<N: ParseNumber>(&self, limit: N) -> Result<N, ParseNumberError> {
        N::parse_with_limits(self, limit)
    }

    fn to_standardized_path(&self) -> String {
        self.replace('\\', "/")
    }

    fn clean_filename(&self) -> String {
        self.trim_matches('"')
            .replace("\\\\", "\\")
            .to_standardized_path()
    }
}
