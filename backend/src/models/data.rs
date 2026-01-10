use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum Data {
    None,
    Some(Value),
}

impl Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Data::None => serializer.serialize_none(),
            Data::Some(value) => serializer.serialize_some(value),
        }
    }
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Option<Value> = Option::deserialize(deserializer)?;
        Ok(value.map(Data::Some).unwrap_or(Data::None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize_data_none() {
        let data = Data::None;
        let serialized = serde_json::to_string(&data).unwrap();
        assert_eq!(serialized, "null");
    }

    #[test]
    fn test_serialize_data_some() {
        let value = json!({ "key": "value" });
        let data = Data::Some(value.clone());
        let serialized = serde_json::to_string(&data).unwrap();
        assert_eq!(serialized, r#"{"key":"value"}"#);
    }

    #[test]
    fn test_deserialize_data_none() {
        let json_str = "null";
        let deserialized: Data = serde_json::from_str(json_str).unwrap();
        assert!(matches!(deserialized, Data::None));
    }

    #[test]
    fn test_deserialize_data_some() {
        let json_str = r#"{"key":"value"}"#;
        let deserialized: Data = serde_json::from_str(json_str).unwrap();
        match deserialized {
            Data::Some(value) => assert_eq!(value, json!({ "key": "value" })),
            Data::None => panic!("Expected Data::Some"),
        }
    }
}


