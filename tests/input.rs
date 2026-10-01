use serde_json::{Value, json};
use zixcel_interaction::InputDeclaration;
fn declaration() -> InputDeclaration {
    serde_json::from_value(json!({"action":{"operation_id":"replace","target":"source:one","contract_revision":"exact:one",
      "availability":{"state":"available"},"expected_revision_required":true,
      "input_schema":{"type":"object","fields":{"f1":{"type":"string","max_length":80},"f2":{"type":"string","choices":["c1","c2"],"max_length":80}},"required":["f1","f2"]}},
      "fields":{"f1":{"label":"Value","sensitive":false},"f2":{"label":"Decision","sensitive":false,"choices":[{"value":"c1","label":"Same label"},{"value":"c2","label":"Same label"}]}}})).unwrap()
}

#[test]
fn exact_choices_fields_and_redacted_errors() {
    let mut d = declaration();
    d.validate().unwrap();
    d.validate_values(&json!({"f1":"日本語","f2":"c2"}))
        .unwrap();
    for input in [
        json!({"f1":"x","f2":"Same label"}),
        json!({"f1":"x","f2":"c2","roleRef":"forged"}),
    ] {
        assert!(d.validate_values(&input).is_err());
    }
    d.fields.get_mut("f1").unwrap().label = "Renamed".into();
    d.validate_values(&json!({"f1":"x","f2":"c1"})).unwrap();
    let secret = "🔐".repeat(4097);
    let error = d
        .validate_values(&json!({"f1":secret,"f2":"c1"}))
        .unwrap_err();
    assert!(!serde_json::to_string(&error).unwrap().contains('🔐'));
}

#[test]
fn declaration_is_closed_and_nonrecursive() {
    let original = serde_json::to_value(declaration()).unwrap();
    for key in ["sourcePath", "component", "initial"] {
        let mut d = original.clone();
        d["fields"]["f1"][key] = json!("private");
        assert!(serde_json::from_value::<InputDeclaration>(d).is_err());
    }
    let mut d = original.clone();
    d["action"]["input_schema"]["fields"]["f1"] = json!({"type":"object","fields":{}});
    assert!(
        serde_json::from_value::<InputDeclaration>(d)
            .unwrap()
            .validate()
            .is_err()
    );
    let mut d = original;
    d["fields"]["f2"]["choices"][0]["value"] = Value::String("wrong".into());
    assert!(
        serde_json::from_value::<InputDeclaration>(d)
            .unwrap()
            .validate()
            .is_err()
    );
}
