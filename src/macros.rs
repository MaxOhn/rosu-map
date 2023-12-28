#[macro_export]
#[doc(hidden)]
macro_rules! section_keys {
    (
        $( #[$meta:meta] )?
        pub enum $name:ident {
            $( $variant:ident, )*
        }
    ) => {
        $( #[$meta] )?
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub enum $name {
            $( $variant, )*
        }

        impl $name {
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $( $name::$variant => stringify!($variant), )*
                }
            }
        }

        impl std::str::FromStr for $name {
            type Err = $crate::section::UnknownKeyError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $( stringify!($variant) => Ok($name::$variant), )*
                    _ => Err($crate::section::UnknownKeyError),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}
