use crate::util::{ParseNumber, ParseNumberError};

pub trait StrExt {
    fn parse_num<N: ParseNumber>(&self) -> Result<N, ParseNumberError>;

    fn parse_with_limits<N: ParseNumber>(&self, limit: N) -> Result<N, ParseNumberError>;

    fn to_standardized_path(&self) -> String;

    fn clean_filename(&self) -> String;
}

impl StrExt for &str {
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
        self.replace("\\\\", "\\")
            .trim_matches('"')
            .to_standardized_path()
    }
}
