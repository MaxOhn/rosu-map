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

macro_rules! thiserror {
	(
		#[error( $desc:tt )]
		$( #[ $error_attribute:meta ] )*
		$vis:vis struct $error_type_name:ident
			$( (
				$(
					$( #[ $tattr:ident ] )?
					$tt:ty
				),* $(,)?
			) )?;
	) => {
		$( #[ $error_attribute ] )*
		$vis struct $error_type_name $( ( $( $tt )* ))?;

		impl ::std::fmt::Display for $error_type_name {
			fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
				formatter.write_str($desc)
			}
		}

		impl ::std::error::Error for $error_type_name {
			fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
				thiserror!( @STRUCTSOURCE self, $( ( $( $tt ),* ) )? )
			}
		}
	};

	(
		$( #[ $error_attribute:meta ] )*
		$vis:vis struct $error_type_name:ident
			$( (
				$(
					$( #[ $tattr:ident ] )?
					$tt:ty
				),* $(,)?
			) )?;
	) => {
		compile_error!("`#[error(\"...\")]` must be the first attribute");
	};

	(
		@STRUCTSOURCE
		$self_:ident,
		(
			#[ $source_or_from:meta ]
			$t1:ty $( , $t_rest:ty )*
		)
	) => {
		Some(&self.0 as _)
    };

    (
		@STRUCTSOURCE
		$self_:ident,
		$( ( $( $tt:ty ),* ) )?
	) => {
		None
	};

	(
		$( #[$error_attribute:meta] )*
		$vis:vis enum $error_type_name:ident {
			$(
				#[error( $desc:tt )]
				$variant_name:ident
					$( {
						$(
							$( #[ $sattr:ident ] )?
							$sf:ident: $st:ty
						),* $(,)?
					} )?
					$( (
						$(
							$( #[ $tattr:ident ] )?
							$tt:ty
						),* $(,)?
					) )?
			),* $(,)?
		}
	) => {
		$( #[ $error_attribute ] )*
        $vis enum $error_type_name {
            $(
                $variant_name
					$( {
						$( $sf: $st ),*
					} )?
					$( (
						$( $tt ),*
					) )?
            ),*
		}

		impl ::std::fmt::Display for $error_type_name {
			fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
				#![allow(irrefutable_let_patterns)]

				$(
					thiserror!(
						@ENUMFMT
						self,
						formatter,
						$desc,
						$variant_name
						$( { $( $sf: $st ),* } )?
						$( ( $( $tt ),* ) )?
					);
				)*

				unreachable!()
			}
		}

		$(
			thiserror!(
				@ENUMFROM
				$error_type_name,
				$variant_name
				$( {
					$(
						$( #[ $sattr ] )?
						$sf: $st
					),*
				} )?
				$( (
					$(
						$( #[ $tattr ] )?
						$tt
					),*
				) )?
			);
		)*

		impl ::std::error::Error for $error_type_name {
			fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
				$(
					thiserror!(
						@ENUMSOURCE
						self,
						$variant_name
						$( { $( $sf: $st ),* } )?
						$( ( $( $tt ),* ) )?
					);
				)*

				None
			}
		}
	};

	( @ENUMFMT $self_:ident, $fmt:ident, $desc:literal, $variant_name:ident ) => {
        if let Self::$variant_name = $self_ {
			return $fmt.write_str($desc);
        }
    };

	( @ENUMFMT $self_:ident, $fmt:ident, $desc:literal, $variant_name:ident ( $t1:ty ) ) => {
        if let Self::$variant_name(a) = $self_ {
            return write!($fmt, concat!($desc, "{0:.0?}"), a);
        }
    };

    ( @ENUMFROM $e:ident, $variant_name:ident ( #[from] $t:ty ) ) => {
        impl From<$t> for $e {
            fn from(x: $t) -> $e {
                $e::$variant_name(x)
            }
        }
    };

    (
		@ENUMFROM
		$e:ident, $variant_name:ident
		$( ( $( $( #[source] )? $tt:ty ),* ) )?
		$( { $( $( #[source] )? $sf:ident: $st:ty ),* } )?
	) => {};

    (
		@ENUMSOURCE
		$self_:ident,
		$variant_name:ident (
			#[ $source_or_from:meta ]
			$t1:ty $( , $t_rest:ty )*
		)
	) => {
        if let Self::$variant_name(x, ..) = $self_ {
            return Some(x as _);
        }
    };

    (
		@ENUMSOURCE
		$self_:ident,
		$variant_name:ident
		$( ( $( $tt:ty ),* ) )?
		$( { $( $sf:ident: $st:ty ),* } )?
	) => {};
}
