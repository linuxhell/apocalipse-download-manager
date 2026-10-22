//! Data-only compatibility map for saved settings from older ADM versions.
//! Runtime identifiers and RPC calls use the native engine exclusively.
use serde_json::Value;
use std::sync::OnceLock;

fn rules() -> &'static Value {
    static RULES: OnceLock<Value> = OnceLock::new();
    RULES.get_or_init(|| {
        serde_json::from_str(include_str!("../legacy-engine-fields.json"))
            .expect("valid embedded migration data")
    })
}

pub fn retired_repository(value: &str) -> bool {
    value.trim().is_empty()
        || rules()["retiredRepositories"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item.as_str() == Some(value))
}

pub fn migrate(value: &mut Value) {
    match value {
        Value::Array(items) => items.iter_mut().for_each(migrate),
        Value::Object(object) => {
            for (old, new) in rules()["keys"].as_object().unwrap() {
                if let Some(value) = object.remove(old) {
                    object.entry(new.as_str().unwrap()).or_insert(value);
                }
            }
            if object
                .get("xtreme_path")
                .and_then(Value::as_str)
                .is_some_and(|path| {
                    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
                    rules()["retiredExecutables"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|item| {
                            item.as_str()
                                .is_some_and(|old| old.eq_ignore_ascii_case(name))
                        })
                })
            {
                object.insert("xtreme_path".into(), Value::Null);
            }
            if object
                .get("engine")
                .and_then(Value::as_str)
                .is_some_and(|name| {
                    rules()["engineValues"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|old| old.as_str() == Some(name))
                })
            {
                object.insert("engine".into(), Value::String("nibiru-xtreme".into()));
            }
            object.values_mut().for_each(migrate);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_fields_migrate_without_changing_destinations_or_current_choices() {
        let mapping = rules()["keys"].as_object().unwrap();
        for (old, new) in mapping {
            let new = new.as_str().unwrap();
            let mut value =
                serde_json::json!({"destination": "D:\\Downloads\\keep.iso", "received": 42});
            value[old] = Value::String("saved".into());
            migrate(&mut value);
            assert_eq!(value[new], "saved");
            assert!(value.get(old).is_none());
            assert_eq!(value["received"], 42);
            assert_eq!(value["destination"], "D:\\Downloads\\keep.iso");
            let mut value = serde_json::json!({});
            value[old] = Value::String("old".into());
            value[new] = Value::String("chosen".into());
            migrate(&mut value);
            assert_eq!(value[new], "chosen");
        }
    }

    #[test]
    fn retired_executable_paths_fall_back_to_native_and_native_paths_survive() {
        for name in rules()["retiredExecutables"].as_array().unwrap() {
            let mut value = serde_json::json!({"xtreme_path": format!("C:\\Tools\\{}", name.as_str().unwrap())});
            migrate(&mut value);
            assert!(value["xtreme_path"].is_null());
        }
        let mut value = serde_json::json!({"xtreme_path": "C:\\Tools\\nibiru-xtreme.exe"});
        migrate(&mut value);
        assert_eq!(value["xtreme_path"], "C:\\Tools\\nibiru-xtreme.exe");
        assert!(!retired_repository("linuxhell/nibiru-xtreme"));
        assert!(!retired_repository("custom/native-engine"));
    }
}
