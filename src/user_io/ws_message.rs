pub mod wm {

    use std::fmt;

    use crate::user_io::deserializer::IncomingDeserializer as is;

    use serde::{self, Deserialize, Serialize};
    use serde_json::Value;
    use ulid::Ulid;

    pub enum Error {
        RateLimit(i64),
        InvalidStructure,
        MessageSize,
        MonthlyBandwidth,
    }

    pub trait MsgString {
        fn msg_string(&self) -> String;
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{}",
                serde_json::to_string(&ErrorMessage::from(self)).unwrap_or_default()
            )
        }
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    #[serde(deny_unknown_fields)]
    pub struct PiBody {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub cache: Option<bool>,
        pub data: Value,
        #[serde(
            default,
            deserialize_with = "is::option_ulid",
            skip_serializing_if = "Option::is_none"
        )]
        pub unique: Option<Ulid>,
    }

    // redis_hash_to_struct!(PiBody);

    impl fmt::Display for PiBody {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", serde_json::to_string(&self).unwrap_or_default())
        }
    }

    impl PiBody {
        #[allow(clippy::missing_const_for_fn)]
        pub fn from_client(body: ClientBody, unique: Option<Ulid>) -> Self {
            Self {
                cache: None,
                data: body.data,
                unique,
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    #[serde(deny_unknown_fields)]
    pub struct ClientBody {
        #[serde(skip_serializing_if = "Option::is_none", skip_deserializing)]
        pub cache: Option<bool>,
        pub data: Value,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "is::option_always_true"
        )]
        pub unique: Option<bool>,
    }

    impl fmt::Display for ClientBody {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", serde_json::to_string(&self).unwrap_or_default())
        }
    }

    impl ClientBody {
        #[allow(clippy::missing_const_for_fn)]
        pub fn from_pi(body: PiBody) -> Self {
            Self {
                cache: None,
                data: body.data,
                unique: None,
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ErrorBody {
        message: String,
        code: u32,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ErrorMessage {
        pub error: ErrorBody,
    }

    impl From<&Error> for ErrorMessage {
        fn from(err: &Error) -> Self {
            let error = match err {
                Error::InvalidStructure => ErrorBody {
                    message: "received data is invalid structure".to_owned(),
                    code: 400,
                },
                Error::RateLimit(limit) => ErrorBody {
                    message: format!("rate limited for {limit} seconds"),
                    code: 429,
                },
                Error::MessageSize => ErrorBody {
                    message: "message size too large".to_owned(),
                    code: 413,
                },
                Error::MonthlyBandwidth => ErrorBody {
                    message: "monthly bandwidth allowance exceeded".to_owned(),
                    code: 509,
                },
            };
            Self { error }
        }
    }
}
