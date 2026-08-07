macro_rules! str_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident => $text:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Debug, ::serde::Serialize, ::serde::Deserialize)]
        $vis enum $name {
            $(
                #[serde(rename = $text)]
                $variant,
            )+
        }

        impl $name {
            /// The exact string this variant is sent as, and parsed from.
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $text,)+
                }
            }

            /// Every variant, in declaration order.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

pub(crate) use str_enum;

#[cfg(test)]
mod tests {
    str_enum! {
        pub enum Example {
            First => "FIRST",
            SecondOne => "SECOND_ONE",
        }
    }

    #[test]
    fn as_str_display_and_serde_agree() {
        for variant in Example::ALL {
            let json = serde_json::to_string(variant).unwrap();

            assert_eq!(json, format!("\"{}\"", variant.as_str()));
            assert_eq!(variant.to_string(), variant.as_str());
            assert_eq!(
                serde_json::from_str::<Example>(&json).unwrap(),
                *variant,
                "round trip failed"
            );
        }
    }
}
