use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

pub fn yaml_to_json_value(yaml: &YamlValue) -> JsonValue {
    match yaml {
        YamlValue::Null => JsonValue::Null,
        YamlValue::Bool(b) => JsonValue::Bool(*b),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsonValue::Number(serde_json::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                JsonValue::Number(
                    serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0)),
                )
            } else {
                JsonValue::Null
            }
        }
        YamlValue::String(s) => JsonValue::String(s.clone()),
        YamlValue::Sequence(seq) => JsonValue::Array(seq.iter().map(yaml_to_json_value).collect()),
        YamlValue::Mapping(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                let key = match k {
                    YamlValue::String(s) => s.clone(),
                    _ => k.as_str().unwrap_or_default().to_string(),
                };
                json_map.insert(key, yaml_to_json_value(v));
            }
            JsonValue::Object(json_map)
        }
        YamlValue::Tagged(_) => todo!(),
    }
}
