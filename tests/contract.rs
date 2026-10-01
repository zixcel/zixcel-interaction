use serde_json::json;
use zixcel_interaction::{
    ActionContract, InvokeRequest, Outcome, ResourceSnapshot, ValueSchema, validate_invoke,
};

fn schema() -> ValueSchema {
    serde_json::from_value(json!({"type":"object", "fields": {
        "enabled": {"type":"boolean"},
        "name": {"type":"string", "min_length":1, "max_length":12},
        "tags": {"type":"array", "max_items":2, "items":{"type":"string", "choices":["a","b"]}}
    }, "required":["enabled", "name"]}))
    .unwrap()
}

#[test]
fn typed_input_preserves_false_and_checks_nested_paths_and_unknown_properties() {
    let s = schema();
    assert!(
        s.validate(&json!({"enabled":false,"name":"Ada"}))
            .is_empty()
    );
    for (input, path) in [
        (json!({"enabled":"false","name":"Ada"}), "/enabled"),
        (json!({"enabled":true,"name":""}), "/name"),
        (json!({"enabled":true,"name":"Ada","admin":true}), "/admin"),
        (json!({"enabled":true,"name":"Ada","tags":["c"]}), "/tags/0"),
        (json!({"name":"Ada"}), "/enabled"),
    ] {
        assert!(
            s.validate(&input).iter().any(|issue| issue.path == path),
            "{input}"
        );
    }
    assert!(
        serde_json::from_value::<ValueSchema>(json!({"type":"string","component":"UInput"}))
            .is_err()
    );
}

fn action() -> ActionContract {
    serde_json::from_value(json!({"operation_id":"update", "target":"item/one",
        "contract_revision":"c1", "input_schema":schema(), "availability":{"state":"available"},
        "expected_revision_required":true}))
    .unwrap()
}

fn request() -> InvokeRequest {
    serde_json::from_value(json!({"operation_id":"update", "target":"item/one", "contract_revision":"c1",
        "input":{"enabled":false,"name":"Ada"}, "expected_revision":"r1", "request_reference":"req1"})).unwrap()
}

#[test]
fn invoke_checks_exact_identity_and_revisions_without_transport_inference() {
    let a = action();
    let r = request();
    assert!(validate_invoke(&a, &r, "r1").is_ok());
    assert_eq!(
        validate_invoke(&a, &r, "r2").unwrap_err().status.as_str(),
        "Conflict"
    );
    let mut wrong = r.clone();
    wrong.target = "item/two".into();
    assert_eq!(
        validate_invoke(&a, &wrong, "r1")
            .unwrap_err()
            .status
            .as_str(),
        "NotFound"
    );
    wrong = r.clone();
    wrong.contract_revision = "old".into();
    assert_eq!(
        validate_invoke(&a, &wrong, "r1")
            .unwrap_err()
            .status
            .as_str(),
        "PreconditionFailed"
    );
    wrong = r.clone();
    wrong.request_reference.clear();
    assert_eq!(
        validate_invoke(&a, &wrong, "r1")
            .unwrap_err()
            .status
            .as_str(),
        "ValidationError"
    );
    wrong = r.clone();
    wrong.input = json!({"enabled":false});
    assert_eq!(
        validate_invoke(&a, &wrong, "r1").unwrap_err().issues[0].path,
        "/name"
    );
}

#[test]
fn all_outcomes_roundtrip_with_typed_details_not_http_status() {
    for status in [
        "Success",
        "ValidationError",
        "Conflict",
        "Forbidden",
        "Unavailable",
        "NotFound",
        "PreconditionFailed",
    ] {
        let value = if status == "Success" {
            json!({"status":status,"value":{"exact":false}})
        } else {
            json!({"status":status,"reason":"test/reason","issues":[]})
        };
        let result: Outcome<serde_json::Value> = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(result).unwrap(), value);
    }
    assert!(serde_json::from_value::<Outcome<serde_json::Value>>(json!({"status":500})).is_err());
    assert!(
        serde_json::from_value::<Outcome<serde_json::Value>>(json!({"status":"Unknown"})).is_err()
    );
}

#[test]
fn resource_visibility_null_and_revision_invariants_are_enforced() {
    let base = json!({"contract":{"resource_id":"one","contract_revision":"c1",
        "value_schema":{"type":"boolean"}, "readable":true,
        "availability":{"state":"available"},"operations":[]},
        "resource_revision":"r1","value":false});
    let resource: ResourceSnapshot = serde_json::from_value(base.clone()).unwrap();
    assert!(resource.validate().is_ok());
    let mut private = resource.clone();
    private.contract.readable = false;
    assert!(private.validate().is_err());
    private.value = None;
    assert!(private.validate().is_ok());
    let mut missing = resource.clone();
    missing.value = None;
    assert!(missing.validate().is_err());
    let mut broken = resource;
    broken.resource_revision.clear();
    assert!(broken.validate().is_err());
    let mut null = base;
    null["contract"]["value_schema"] = json!({"type":"null"});
    null["value"] = json!(null);
    let parsed: ResourceSnapshot = serde_json::from_value(null).unwrap();
    assert_eq!(parsed.value, Some(serde_json::Value::Null));
    assert!(parsed.validate().is_ok());
}
