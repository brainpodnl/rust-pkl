use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{collections::HashMap, fs, path::Path};
use typed_builder::TypedBuilder;

use crate::{errors::ProjectError, protocol::Message};

// Client Request Messages

#[skip_serializing_none]
#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
#[builder(field_defaults(default, setter(strip_option)))]
pub struct CreateEvaluatorRequest {
    #[builder(!default, setter(!strip_option))]
    pub request_id: u64,
    pub allowed_modules: Option<Vec<String>>,
    pub allowed_resources: Option<Vec<String>>,
    pub client_module_readers: Option<Vec<ClientModuleReader>>,
    pub client_resource_readers: Option<Vec<ClientResourceReader>>,
    pub module_paths: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub properties: Option<HashMap<String, String>>,
    pub timeout_seconds: Option<i64>,
    pub root_dir: Option<String>,
    pub cache_dir: Option<String>,
    pub output_format: Option<String>,
    pub project: Option<Project>,
    pub http: Option<Http>,
}

impl Message for CreateEvaluatorRequest {
    const CODE: u64 = 0x20;
}

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct CloseEvaluator {
    pub evaluator_id: i64,
}

impl Message for CloseEvaluator {
    const CODE: u64 = 0x22;
}

#[skip_serializing_none]
#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateRequest {
    pub request_id: u64,
    pub evaluator_id: i64,
    pub module_uri: String,
    #[builder(default, setter(strip_option))]
    pub module_text: Option<String>,
    #[builder(default, setter(strip_option))]
    pub expr: Option<String>,
}

impl Message for EvaluateRequest {
    const CODE: u64 = 0x23;
}

// Client Response Messages

#[skip_serializing_none]
#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ReadResourceResponse {
    pub request_id: u64,
    pub evaluator_id: i64,
    #[builder(default, setter(strip_option))]
    pub contents: Option<Vec<u8>>, // Binary data
    #[builder(default, setter(strip_option))]
    pub error: Option<String>,
}

impl Message for ReadResourceResponse {
    const CODE: u64 = 0x27;
}

#[skip_serializing_none]
#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ReadModuleResponse {
    pub request_id: u64,
    pub evaluator_id: i64,
    #[builder(default, setter(strip_option))]
    pub contents: Option<String>,
    #[builder(default, setter(strip_option))]
    pub error: Option<String>,
}

impl Message for ReadModuleResponse {
    const CODE: u64 = 0x29;
}

#[skip_serializing_none]
#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesResponse {
    pub request_id: u64,
    pub evaluator_id: i64,
    #[builder(default, setter(strip_option))]
    pub path_elements: Option<Vec<PathElement>>,
    #[builder(default, setter(strip_option))]
    pub error: Option<String>,
}

impl Message for ListResourcesResponse {
    const CODE: u64 = 0x2b;
}

#[skip_serializing_none]
#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ListModulesResponse {
    pub request_id: u64,
    pub evaluator_id: i64,
    #[builder(default, setter(strip_option))]
    pub path_elements: Option<Vec<PathElement>>,
    #[builder(default, setter(strip_option))]
    pub error: Option<String>,
}

impl Message for ListModulesResponse {
    const CODE: u64 = 0x2d;
}

#[skip_serializing_none]
#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct InitializeModuleReaderResponse {
    pub request_id: u64,
    #[builder(default, setter(strip_option))]
    pub spec: Option<ClientModuleReader>,
}

impl Message for InitializeModuleReaderResponse {
    const CODE: u64 = 0x2f;
}

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResourceReaderResponse {
    pub request_id: u64,
    #[builder(default, setter(strip_option))]
    pub spec: Option<ClientResourceReader>,
}

impl Message for InitializeResourceReaderResponse {
    const CODE: u64 = 0x31;
}

// Supporting Types

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ClientResourceReader {
    pub scheme: String,
    pub has_hierarchical_uris: bool,
    pub is_globbable: bool,
}

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ClientModuleReader {
    pub scheme: String,
    pub has_hierarchical_uris: bool,
    pub is_globbable: bool,
    pub is_local: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectType {
    Local,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    #[serde(rename = "type")]
    pub project_type: ProjectType,
    #[builder(default, setter(strip_option))]
    pub package_uri: Option<String>,
    pub project_file_uri: String,
    pub dependencies: HashMap<String, ProjectDependency>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Dependencies {
    resolved_dependencies: HashMap<String, ProjectDependency>,
}

impl Project {
    pub fn from_path(root_dir: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let project_file = root_dir.as_ref().join("PklProject");
        let contents = fs::read(root_dir.as_ref().join("PklProject.deps.json"))?;
        let deps: Dependencies = serde_json::from_slice(&contents)?;

        Ok(Project::builder()
            .project_type(ProjectType::Local)
            .project_file_uri(format!("file://{}", project_file.display()))
            .dependencies(
                deps.resolved_dependencies
                    .into_iter()
                    .filter_map(|(uri, dep)| {
                        if let Some((path, _)) = uri.rsplit_once('@') {
                            if let Some((_, name)) = path.rsplit_once('/') {
                                return Some((name.to_string(), dep));
                            }
                        }

                        None
                    })
                    .collect(),
            )
            .build())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ProjectDependency {
    Local(Project),
    Remote(RemoteDependency),
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct RemoteDependency {
    #[serde(alias = "uri")]
    #[builder(default, setter(strip_option))]
    pub package_uri: Option<String>,
    #[builder(default, setter(strip_option))]
    pub checksums: Option<Checksums>,
}

#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct Checksums {
    pub sha256: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct Http {
    #[builder(default, setter(strip_option))]
    pub ca_certificates: Option<Vec<u8>>,
    #[builder(default, setter(strip_option))]
    pub proxy: Option<Proxy>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    #[builder(default, setter(strip_option))]
    pub address: Option<String>,
    pub no_proxy: Vec<String>,
}

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct PathElement {
    pub name: String,
    pub is_directory: bool,
}
