// Custom Serialize/Deserialize implementations for AgentMessage (Bytes field)

impl Serialize for AgentMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AgentMessage", 2)?;
        state.serialize_field("header", &self.header)?;
        state.serialize_field("payload", &self.payload.to_vec())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for AgentMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};

        struct AgentMessageVisitor;

        impl<'de> Visitor<'de> for AgentMessageVisitor {
            type Value = AgentMessage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AgentMessage")
            }

            fn visit_map<V>(self, mut map: V) -> Result<AgentMessage, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut header = None;
                let mut payload = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "header" => {
                            header = Some(map.next_value()?);
                        }
                        "payload" => {
                            let bytes: Vec<u8> = map.next_value()?;
                            payload = Some(Bytes::from(bytes));
                        }
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                Ok(AgentMessage {
                    header: header.ok_or_else(|| de::Error::missing_field("header"))?,
                    payload: payload.ok_or_else(|| de::Error::missing_field("payload"))?,
                })
            }
        }

        deserializer.deserialize_struct("AgentMessage", &["header", "payload"], AgentMessageVisitor)
    }
}
