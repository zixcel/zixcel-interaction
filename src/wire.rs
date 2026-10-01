use crate::{Issue, ValueSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CONTRACT: &str = "zixcel://interaction/v1";
pub const MAX_MESSAGE_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum Availability {
    Available,
    Unavailable { reason: String },
    Forbidden { reason: String },
    PreconditionFailed { reason: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionContract {
    pub operation_id: String,
    pub target: String,
    pub contract_revision: String,
    pub input_schema: ValueSchema,
    pub availability: Availability,
    pub expected_revision_required: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceContract {
    pub resource_id: String,
    pub contract_revision: String,
    pub value_schema: ValueSchema,
    pub readable: bool,
    pub availability: Availability,
    pub operations: Vec<ActionContract>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceSnapshot {
    pub contract: ResourceContract,
    pub resource_revision: String,
    /// None means unavailable/unreadable, distinct from an available JSON null.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_value"
    )]
    pub value: Option<Value>,
}

fn present_value<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Value>, D::Error> {
    Value::deserialize(deserializer).map(Some)
}

impl ResourceSnapshot {
    pub fn validate(&self) -> Result<(), Failure> {
        let contract = &self.contract;
        if ![
            &contract.resource_id,
            &contract.contract_revision,
            &self.resource_revision,
        ]
        .into_iter()
        .all(|s| valid_reference(s))
            || contract.operations.len() > 128
        {
            return Err(Failure::new(
                FailureStatus::ValidationError,
                "resource/contract/invalid",
            ));
        }
        let visible = contract.readable && matches!(contract.availability, Availability::Available);
        if !contract.value_schema.is_valid() {
            return Err(Failure::new(
                FailureStatus::ValidationError,
                "resource/schema/invalid",
            ));
        }
        if visible != self.value.is_some() {
            return Err(Failure::new(
                FailureStatus::ValidationError,
                "resource/visibility/invalid",
            ));
        }
        if let Some(value) = &self.value {
            let issues = contract.value_schema.validate(value);
            if !issues.is_empty() {
                return Err(Failure {
                    status: FailureStatus::ValidationError,
                    reason: "resource/value/invalid".into(),
                    issues,
                });
            }
        }
        let mut ids = std::collections::BTreeSet::new();
        for operation in &contract.operations {
            if operation.target != contract.resource_id
                || !valid_reference(&operation.operation_id)
                || !valid_reference(&operation.contract_revision)
                || !operation.input_schema.is_valid()
                || !ids.insert(&operation.operation_id)
            {
                return Err(Failure::new(
                    FailureStatus::ValidationError,
                    "resource/operation/invalid",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvokeRequest {
    pub operation_id: String,
    pub target: String,
    pub contract_revision: String,
    pub input: Value,
    pub expected_revision: Option<String>,
    /// Scoped by the authenticated handler identity; not authorization supplied by the client.
    pub request_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadRequest {
    pub resources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscribeRequest {
    pub resources: Vec<String>,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub resource_id: String,
    pub resource_revision: String,
    pub contract_revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ChangeBatch {
    Changes {
        after: String,
        cursor: String,
        changes: Vec<Change>,
    },
    /// Consumer must reread one coherent snapshot; never apply an incomplete delta.
    Reset { cursor: String, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureStatus {
    ValidationError,
    Conflict,
    Forbidden,
    Unavailable,
    NotFound,
    PreconditionFailed,
}

impl FailureStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ValidationError => "ValidationError",
            Self::Conflict => "Conflict",
            Self::Forbidden => "Forbidden",
            Self::Unavailable => "Unavailable",
            Self::NotFound => "NotFound",
            Self::PreconditionFailed => "PreconditionFailed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Failure {
    pub status: FailureStatus,
    pub reason: String,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuccessStatus {
    Success,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Outcome<T> {
    Success { status: SuccessStatus, value: T },
    Failure(Failure),
}

impl<T> Outcome<T> {
    pub fn success(value: T) -> Self {
        Self::Success {
            status: SuccessStatus::Success,
            value,
        }
    }
}

impl Failure {
    pub fn new(status: FailureStatus, reason: impl Into<String>) -> Self {
        Self {
            status,
            reason: reason.into(),
            issues: Vec::new(),
        }
    }
}

/// Structural checks only. The handler must authorize scope, perform atomic CAS,
/// and persist retry receipts with its facts. This function does not execute an action.
pub fn validate_invoke(
    action: &ActionContract,
    request: &InvokeRequest,
    current_revision: &str,
) -> Result<(), Failure> {
    use FailureStatus as F;
    if ![
        &request.operation_id,
        &request.target,
        &request.contract_revision,
        &request.request_reference,
    ]
    .into_iter()
    .all(|s| valid_reference(s))
        || request
            .expected_revision
            .as_ref()
            .is_some_and(|s| !valid_reference(s))
    {
        return Err(Failure::new(
            F::ValidationError,
            "request/reference/invalid",
        ));
    }
    if action.operation_id != request.operation_id || action.target != request.target {
        return Err(Failure::new(F::NotFound, "operation/not/found"));
    }
    match &action.availability {
        Availability::Available => {}
        Availability::Unavailable { reason } => return Err(Failure::new(F::Unavailable, reason)),
        Availability::Forbidden { reason } => return Err(Failure::new(F::Forbidden, reason)),
        Availability::PreconditionFailed { reason } => {
            return Err(Failure::new(F::PreconditionFailed, reason));
        }
    }
    if action.contract_revision != request.contract_revision {
        return Err(Failure::new(
            F::PreconditionFailed,
            "contract/revision/changed",
        ));
    }
    if action.expected_revision_required && request.expected_revision.is_none() {
        return Err(Failure::new(
            F::PreconditionFailed,
            "resource/revision/required",
        ));
    }
    if request
        .expected_revision
        .as_ref()
        .is_some_and(|s| s != current_revision)
    {
        return Err(Failure::new(F::Conflict, "resource/revision/changed"));
    }
    let issues = action.input_schema.validate(&request.input);
    if !issues.is_empty() {
        return Err(Failure {
            status: F::ValidationError,
            reason: "input/invalid".into(),
            issues,
        });
    }
    Ok(())
}

pub fn valid_reference(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}

pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, Failure> {
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(Failure::new(
            FailureStatus::ValidationError,
            "message/limit",
        ));
    }
    serde_json::from_slice(bytes)
        .map_err(|_| Failure::new(FailureStatus::ValidationError, "message/invalid"))
}
