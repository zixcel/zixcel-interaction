//! Owner-authored presentation metadata over ActionContract. No semantic or source binding.
use crate::{
    ActionContract, Availability, Failure, FailureStatus, ResourceContract, ResourceSnapshot,
    ValueSchema, valid_reference,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_INPUT_FIELDS: usize = 32;
pub const MAX_INPUT_CHOICES: usize = 64;
pub const MAX_INPUT_STRING_BYTES: usize = 4096;
pub const MAX_INPUT_BYTES: usize = 16384;
pub const MAX_INPUT_CONTRACT_BYTES: usize = 32768;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputChoice {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputField {
    pub label: String,
    pub sensitive: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<InputChoice>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputDeclaration {
    pub action: ActionContract,
    /// Opaque field references, not source paths. All members are editable.
    /// Fixed owner values must never be included in this declaration.
    pub fields: BTreeMap<String, InputField>,
}

fn invalid(reason: &str) -> Failure {
    Failure::new(FailureStatus::ValidationError, reason)
}

struct ByteBudget(usize);
impl std::io::Write for ByteBudget {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_sub(bytes.len())
            .ok_or_else(|| std::io::Error::other("input limit"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn bounded(value: &impl Serialize, maximum: usize) -> bool {
    serde_json::to_writer(&mut ByteBudget(maximum), value).is_ok()
}

impl InputDeclaration {
    pub fn validate(&self) -> Result<(), Failure> {
        let bad = || invalid("input/declaration/invalid");
        if !bounded(self, MAX_INPUT_CONTRACT_BYTES) {
            return Err(bad());
        }
        ResourceSnapshot {
            contract: ResourceContract {
                resource_id: self.action.target.clone(),
                contract_revision: self.action.contract_revision.clone(),
                value_schema: ValueSchema::Null,
                readable: false,
                availability: Availability::Available,
                operations: vec![self.action.clone()],
            },
            resource_revision: self.action.contract_revision.clone(),
            value: None,
        }
        .validate()
        .map_err(|_| bad())?;
        let ValueSchema::Object { fields, .. } = &self.action.input_schema else {
            return Err(bad());
        };
        if fields.len() > MAX_INPUT_FIELDS || fields.len() != self.fields.len() {
            return Err(bad());
        }
        for (id, field) in &self.fields {
            if !valid_reference(id) || !valid_reference(&field.label) {
                return Err(bad());
            }
            match fields.get(id) {
                Some(
                    schema @ ValueSchema::String {
                        max_length,
                        choices,
                        ..
                    },
                ) => {
                    if *max_length > MAX_INPUT_STRING_BYTES
                        || choices.len() > MAX_INPUT_CHOICES
                        || choices.iter().collect::<BTreeSet<_>>().len() != choices.len()
                        || choices.iter().any(|c| {
                            c.len() > MAX_INPUT_STRING_BYTES
                                || !schema.validate(&Value::String(c.clone())).is_empty()
                        })
                    {
                        return Err(bad());
                    }
                    if choices.is_empty() {
                        if field.choices.is_some() {
                            return Err(bad());
                        }
                    } else {
                        let Some(presentation) = &field.choices else {
                            return Err(bad());
                        };
                        if presentation.len() != choices.len()
                            || presentation
                                .iter()
                                .zip(choices)
                                .any(|(p, c)| &p.value != c || !valid_reference(&p.label))
                        {
                            return Err(bad());
                        }
                    }
                }
                Some(
                    ValueSchema::Boolean | ValueSchema::Number { .. } | ValueSchema::Integer { .. },
                ) if field.choices.is_none() => (),
                _ => return Err(bad()),
            }
        }
        Ok(())
    }

    /// Validation errors contain reasons only, never submitted values.
    pub fn validate_values(&self, values: &Value) -> Result<(), Failure> {
        self.validate()?;
        if !bounded(values, MAX_INPUT_BYTES) {
            return Err(invalid("input/limit"));
        }
        let Some(values_object) = values.as_object() else {
            return Err(invalid("input/limit"));
        };
        if values_object
            .values()
            .any(|v| v.as_str().is_some_and(|s| s.len() > MAX_INPUT_STRING_BYTES))
        {
            return Err(invalid("input/limit"));
        }
        if !self.action.input_schema.validate(values).is_empty() {
            return Err(invalid("input/invalid"));
        }
        Ok(())
    }
}
