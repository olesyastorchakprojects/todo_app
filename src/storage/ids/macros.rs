macro_rules! define_uuid_id {
    ($id_type:ident) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
        #[repr(transparent)]
        pub struct $id_type(uuid::Uuid);

        impl $id_type {
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
        }

        impl Default for $id_type {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $id_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $id_type {
            type Err = $crate::storage::StorageError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(uuid::Uuid::parse_str(s).map_err(
                    $crate::storage::StorageError::ParseIdFromString,
                )?))
            }
        }

        impl<'de> serde::Deserialize<'de> for $id_type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                uuid::Uuid::parse_str(&s)
                    .map(Self)
                    .map_err(serde::de::Error::custom)
            }
        }

        impl From<uuid::Uuid> for $id_type {
            fn from(value: uuid::Uuid) -> Self {
                Self(value)
            }
        }

        impl From<$id_type> for uuid::Uuid {
            fn from(value: $id_type) -> Self {
                value.0
            }
        }

        impl bincode::Encode for $id_type {
            fn encode<E: bincode::enc::Encoder>(
                &self,
                encoder: &mut E,
            ) -> Result<(), bincode::error::EncodeError> {
                self.0.as_bytes().encode(encoder)
            }
        }

        impl<Context> bincode::Decode<Context> for $id_type {
            fn decode<D: bincode::de::Decoder>(
                decoder: &mut D,
            ) -> Result<Self, bincode::error::DecodeError> {
                let bytes: [u8; 16] = bincode::Decode::decode(decoder)?;
                Ok(Self(uuid::Uuid::from_bytes(bytes)))
            }
        }

        impl<'de, Context> bincode::BorrowDecode<'de, Context> for $id_type {
            fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
                decoder: &mut D,
            ) -> Result<Self, bincode::error::DecodeError> {
                let bytes: [u8; 16] = bincode::Decode::decode(decoder)?;
                Ok(Self(uuid::Uuid::from_bytes(bytes)))
            }
        }
    };
}
