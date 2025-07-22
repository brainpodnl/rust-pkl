use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    fs,
    path::{Path, PathBuf},
};

use crate::{errors::ProjectError, protocol::Message};

#[derive(Debug)]
pub enum Uri {
    File(PathBuf),
    Url(String),
}

impl Default for Uri {
    fn default() -> Self {
        Uri::File("/dev/null".into())
    }
}

impl<'de> Deserialize<'de> for Uri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s.starts_with("file://") {
            Ok(Uri::File(s.trim_start_matches("file://").into()))
        } else {
            Ok(Uri::Url(s))
        }
    }
}

impl Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Uri::File(path) => {
                write!(f, "file://{}", path.to_str().unwrap_or_default())
            }
            Uri::Url(url) => write!(f, "{url}"),
        }
    }
}

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

// Client Request Messages

#[skip_serializing_none]
#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEvaluatorRequest<'a> {
    pub request_id: u64,
    pub allowed_modules: Option<&'a [String]>,
    pub allowed_resources: Option<&'a [String]>,
    pub client_module_readers: Option<&'a [ClientModuleReader]>,
    pub client_resource_readers: Option<&'a [ClientResourceReader]>,
    pub module_paths: Option<&'a [String]>,
    pub env: Option<&'a HashMap<String, String>>,
    pub properties: Option<HashMap<String, String>>,
    pub timeout_seconds: Option<i64>,
    pub root_dir: Option<&'a str>,
    pub cache_dir: Option<&'a str>,
    pub output_format: Option<&'a str>,
    pub project: Option<&'a Project>,
    pub http: Option<&'a Http>,
}

impl<'a> Message for CreateEvaluatorRequest<'a> {
    const CODE: u64 = 0x20;
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseEvaluator {
    pub evaluator_id: i64,
}

impl Message for CloseEvaluator {
    const CODE: u64 = 0x22;
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateRequest<'a> {
    pub request_id: u64,
    pub evaluator_id: i64,
    pub module_uri: Uri,
    pub module_text: Option<&'a str>,
    pub expr: Option<&'a str>,
}

impl<'a> Message for EvaluateRequest<'a> {
    const CODE: u64 = 0x23;
}

// Client Response Messages

#[skip_serializing_none]
#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadResourceResponse<'a> {
    pub request_id: u64,
    pub evaluator_id: i64,
    pub contents: Option<&'a [u8]>, // Binary data
    pub error: Option<&'a str>,
}

impl<'a> Message for ReadResourceResponse<'a> {
    const CODE: u64 = 0x27;
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadModuleResponse<'a> {
    pub request_id: u64,
    pub evaluator_id: i64,
    pub contents: Option<&'a str>,
    pub error: Option<&'a str>,
}

impl<'a> Message for ReadModuleResponse<'a> {
    const CODE: u64 = 0x29;
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesResponse<'a> {
    pub request_id: u64,
    pub evaluator_id: i64,
    pub path_elements: Option<&'a [PathElement]>,
    pub error: Option<&'a str>,
}

impl<'a> Message for ListResourcesResponse<'a> {
    const CODE: u64 = 0x2b;
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListModulesResponse<'a> {
    pub request_id: u64,
    pub evaluator_id: i64,
    pub path_elements: Option<&'a [PathElement]>,
    pub error: Option<&'a str>,
}

impl<'a> Message for ListModulesResponse<'a> {
    const CODE: u64 = 0x2d;
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeModuleReaderResponse<'a> {
    pub request_id: u64,
    pub spec: Option<&'a ClientModuleReader>,
}

impl<'a> Message for InitializeModuleReaderResponse<'a> {
    const CODE: u64 = 0x2f;
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResourceReaderResponse<'a> {
    pub request_id: u64,
    pub spec: Option<&'a ClientResourceReader>,
}

impl<'a> Message for InitializeResourceReaderResponse<'a> {
    const CODE: u64 = 0x31;
}

// Supporting Types

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientResourceReader {
    pub scheme: String,
    pub has_hierarchical_uris: bool,
    pub is_globbable: bool,
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientModuleReader {
    pub scheme: String,
    pub has_hierarchical_uris: bool,
    pub is_globbable: bool,
    pub is_local: bool,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectType {
    #[default]
    Local,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    #[serde(rename = "type")]
    pub ty: ProjectType,
    pub package_uri: Option<Uri>,
    pub project_file_uri: Uri,
    pub dependencies: HashMap<String, ProjectDependency>,
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Dependencies {
    resolved_dependencies: HashMap<String, ProjectDependency>,
}

impl Project {
    pub fn from_path(root_dir: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let project_file = root_dir.as_ref().join("PklProject");
        let contents = fs::read(root_dir.as_ref().join("PklProject.deps.json"))?;
        let deps: Dependencies = serde_json::from_slice(&contents)?;
        let mut project = Project::default();

        project.ty = ProjectType::Local;
        project.project_file_uri = Uri::File(project_file);
        project.dependencies = deps
            .resolved_dependencies
            .into_iter()
            .filter_map(|(uri, dep)| {
                if let Some((path, _)) = uri.rsplit_once('@') {
                    if let Some((_, name)) = path.rsplit_once('/') {
                        return Some((name.to_string(), dep));
                    }
                }

                None
            })
            .collect();

        Ok(project)
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
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteDependency {
    #[serde(alias = "uri")]
    pub package_uri: Option<Uri>,
    pub checksums: Option<Checksums>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checksums {
    pub sha256: String,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Http {
    pub ca_certificates: Option<Vec<u8>>,
    pub proxy: Option<Proxy>,
}

#[skip_serializing_none]
#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    pub address: Option<String>,
    pub no_proxy: Vec<String>,
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathElement {
    pub name: String,
    pub is_directory: bool,
}
