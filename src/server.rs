use std::collections::HashMap;

use serde::Deserialize;
use serde_with::skip_serializing_none;

use crate::{errors::ValueError, protocol::Message};

#[derive(Debug)]
pub struct Object {
    pub class_name: String,
    pub module_uri: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug)]
pub enum Value {
    Null,
    Int(i64),
    Uint(u64),
    Float(f64),
    Bool(bool),
    String(String),
    Function,
    Object(Object),
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Mapping(Vec<(Value, Value)>),
}

impl TryFrom<Value> for String {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(ValueError::UnexpectedValue),
        }
    }
}

pub enum Response {
    CreateEvaluator(CreateEvaluatorResponse),
    Evaluate(EvaluateResponse),
    Log(Log),
    ReadResource(ReadResourceRequest),
    ReadModule(ReadModuleRequest),
    ListResources(ListResourcesRequest),
    ListModules(ListModulesRequest),
    InitializeModuleReader(InitializeModuleReaderRequest),
    InitializeResourceReader(InitializeResourceReaderRequest),
    CloseExternalProcess(CloseExternalProcess),
}

macro_rules! impl_from {
    ($(($name:ident, $ty:ident)),+) => {
        impl Response {
            pub fn name(&self) -> &'static str {
                match self {
                    $(Response::$name(_) => stringify!($name),)+
                }
            }
        }

        $(
            impl From<$ty> for Response {
                fn from(value: $ty) -> Self {
                    Response::$name(value)
                }
            }

            impl TryFrom<Response> for $ty {
                type Error = crate::errors::Error;

                fn try_from(response: Response) -> Result<Self, Self::Error> {
                    if let Response::$name(inner) = response {
                        Ok(inner)
                    } else {
                        Err(crate::errors::Error::InvalidResponse(response.name()))
                    }
                }
            }
        )+
    };
}

impl_from!(
    (CreateEvaluator, CreateEvaluatorResponse),
    (Evaluate, EvaluateResponse),
    (Log, Log),
    (ReadResource, ReadResourceRequest),
    (ReadModule, ReadModuleRequest),
    (ListResources, ListResourcesRequest),
    (ListModules, ListModulesRequest),
    (InitializeModuleReader, InitializeModuleReaderRequest),
    (InitializeResourceReader, InitializeResourceReaderRequest),
    (CloseExternalProcess, CloseExternalProcess)
);

// Server Response Messages

#[skip_serializing_none]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEvaluatorResponse {
    pub request_id: u64,
    pub evaluator_id: Option<i64>,
    pub error: Option<String>,
}

impl Message for CreateEvaluatorResponse {
    const CODE: u64 = 0x21;
}

#[skip_serializing_none]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateResponse {
    pub request_id: u64,
    pub evaluator_id: i64,
    pub result: Option<Vec<u8>>, // Binary data (Pkl Binary Encoding)
    pub error: Option<String>,
}

impl Message for EvaluateResponse {
    const CODE: u64 = 0x24;
}

// Server One Way Messages

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub evaluator_id: i64,
    pub level: i64, // 0: trace, 1: warn
    pub message: String,
    pub frame_uri: String,
}

impl Message for Log {
    const CODE: u64 = 0x25;
}

// Server Request Messages

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadResourceRequest {
    pub request_id: u64,
    pub evaluator_id: i64,
    pub uri: String,
}

impl Message for ReadResourceRequest {
    const CODE: u64 = 0x26;
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadModuleRequest {
    pub request_id: u64,
    pub evaluator_id: i64,
    pub uri: String,
}

impl Message for ReadModuleRequest {
    const CODE: u64 = 0x28;
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesRequest {
    pub request_id: u64,
    pub evaluator_id: i64,
    pub uri: String,
}

impl Message for ListResourcesRequest {
    const CODE: u64 = 0x2a;
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListModulesRequest {
    pub request_id: u64,
    pub evaluator_id: i64,
    pub uri: String,
}

impl Message for ListModulesRequest {
    const CODE: u64 = 0x2c;
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeModuleReaderRequest {
    pub request_id: u64,
    pub scheme: String,
}

impl Message for InitializeModuleReaderRequest {
    const CODE: u64 = 0x2e;
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResourceReaderRequest {
    pub request_id: u64,
    pub scheme: String,
}

impl Message for InitializeResourceReaderRequest {
    const CODE: u64 = 0x30;
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseExternalProcess {
    // This message has no properties according to the documentation
}

impl Message for CloseExternalProcess {
    const CODE: u64 = 0x32;
}
