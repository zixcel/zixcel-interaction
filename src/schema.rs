use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Issue {
    /// RFC 6901 JSON pointer, not a UI field identifier.
    pub path: String,
    pub reason: String,
}

/// A deliberately bounded value algebra, not a partial JSON Schema interpreter.
/// No presentation, semantic definition, transport or arbitrary executable validator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ValueSchema {
    Null,
    Boolean,
    String {
        #[serde(default)]
        min_length: usize,
        #[serde(default = "default_length")]
        max_length: usize,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        choices: Vec<String>,
    },
    Number {
        minimum: f64,
        maximum: f64,
    },
    Integer {
        minimum: i64,
        maximum: i64,
    },
    Array {
        items: Box<ValueSchema>,
        max_items: usize,
    },
    Object {
        fields: BTreeMap<String, ValueSchema>,
        #[serde(default)]
        required: Vec<String>,
    },
}

const fn default_length() -> usize {
    4096
}

impl ValueSchema {
    pub fn is_valid(&self) -> bool {
        self.check_schema(0, &mut 4096)
    }

    fn check_schema(&self, depth: usize, budget: &mut usize) -> bool {
        if depth > 16 || *budget == 0 {
            return false;
        }
        *budget -= 1;
        match self {
            Self::Null | Self::Boolean => true,
            Self::String {
                min_length,
                max_length,
                choices,
            } => min_length <= max_length && *max_length <= 65536 && choices.len() <= 256,
            Self::Number { minimum, maximum } => {
                minimum.is_finite() && maximum.is_finite() && minimum <= maximum
            }
            Self::Integer { minimum, maximum } => {
                minimum <= maximum
                    && *minimum >= -9_007_199_254_740_991
                    && *maximum <= 9_007_199_254_740_991
            }
            Self::Array { items, max_items } => {
                *max_items <= 1024 && items.check_schema(depth + 1, budget)
            }
            Self::Object { fields, required } => {
                fields.len() <= 128
                    && required.len() <= fields.len()
                    && required
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        == required.len()
                    && required.iter().all(|key| fields.contains_key(key))
                    && fields
                        .values()
                        .all(|field| field.check_schema(depth + 1, budget))
            }
        }
    }

    pub fn validate(&self, value: &Value) -> Vec<Issue> {
        let mut issues = Vec::new();
        if !self.is_valid() {
            issue(&mut issues, "", "schema/invalid");
            return issues;
        }
        let mut budget = 4096;
        self.visit(value, "", 0, &mut budget, &mut issues);
        issues
    }

    fn visit(
        &self,
        value: &Value,
        path: &str,
        depth: usize,
        budget: &mut usize,
        issues: &mut Vec<Issue>,
    ) {
        if issues.len() >= 64 {
            return;
        }
        if depth > 16 || *budget == 0 {
            issue(issues, path, "limit");
            return;
        }
        *budget -= 1;
        match self {
            Self::Null if value.is_null() => {}
            Self::Boolean if value.is_boolean() => {}
            Self::String {
                min_length,
                max_length,
                choices,
            } => {
                if min_length > max_length || *max_length > 65536 || choices.len() > 256 {
                    issue(issues, path, "schema/invalid");
                    return;
                }
                match value.as_str() {
                    Some(s)
                        if (*min_length..=*max_length).contains(&s.chars().count())
                            && (choices.is_empty() || choices.iter().any(|c| c == s)) => {}
                    _ => issue(issues, path, "string/constraint"),
                }
            }
            Self::Integer { minimum, maximum } => {
                if minimum > maximum {
                    issue(issues, path, "schema/invalid");
                } else if !value
                    .as_i64()
                    .is_some_and(|v| (*minimum..=*maximum).contains(&v))
                {
                    issue(issues, path, "integer/constraint");
                }
            }
            Self::Number { minimum, maximum } => {
                if !minimum.is_finite() || !maximum.is_finite() || minimum > maximum {
                    issue(issues, path, "schema/invalid");
                } else if !value
                    .as_f64()
                    .is_some_and(|v| (*minimum..=*maximum).contains(&v))
                {
                    issue(issues, path, "number/constraint");
                }
            }
            Self::Array { items, max_items } => {
                if *max_items > 1024 {
                    issue(issues, path, "schema/invalid");
                    return;
                }
                let Some(array) = value.as_array() else {
                    issue(issues, path, "type");
                    return;
                };
                if array.len() > *max_items {
                    issue(issues, path, "array/limit");
                    return;
                }
                for (index, item) in array.iter().enumerate() {
                    items.visit(item, &format!("{path}/{index}"), depth + 1, budget, issues);
                }
            }
            Self::Object { fields, required } => {
                if fields.len() > 128
                    || required.len() > fields.len()
                    || required.iter().any(|key| !fields.contains_key(key))
                {
                    issue(issues, path, "schema/invalid");
                    return;
                }
                let Some(object) = value.as_object() else {
                    issue(issues, path, "type");
                    return;
                };
                if object.len() > 128 {
                    issue(issues, path, "object/limit");
                    return;
                }
                for key in required {
                    if !object.contains_key(key) {
                        issue(issues, &pointer(path, key), "required");
                    }
                }
                for (key, item) in object {
                    let next = pointer(path, key);
                    if let Some(schema) = fields.get(key) {
                        schema.visit(item, &next, depth + 1, budget, issues);
                    } else {
                        issue(issues, &next, "unknown");
                    }
                }
            }
            _ => issue(issues, path, "type"),
        }
    }
}

fn pointer(parent: &str, key: &str) -> String {
    format!("{parent}/{}", key.replace('~', "~0").replace('/', "~1"))
}
fn issue(issues: &mut Vec<Issue>, path: &str, reason: &str) {
    if issues.len() < 64 {
        issues.push(Issue {
            path: path.into(),
            reason: reason.into(),
        });
    }
}
