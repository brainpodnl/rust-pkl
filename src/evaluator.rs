use std::io::Cursor;

use crate::{
    client::{CreateEvaluatorRequest, EvaluateRequest, Project},
    decoder::Decoder,
    errors::{Error, PklError},
    protocol::Protocol,
    server::Value,
};

pub struct Evaluator {
    request_id: u64,
    proto: Protocol,
}

pub struct EvalOpts {
    pub allowed_modules: Vec<String>,
    pub allowed_resources: Vec<String>,
    pub output_format: String,
    pub project: Option<Project>,
}

impl Default for EvalOpts {
    fn default() -> Self {
        Self {
            allowed_modules: vec!["pkl:".to_string()],
            allowed_resources: vec![],
            output_format: "pkl".to_string(),
            project: None,
        }
    }
}

impl Evaluator {
    pub fn new(proto: Protocol) -> Self {
        Self {
            proto,
            request_id: 0,
        }
    }

    fn gen_request_id(&mut self) -> u64 {
        let request_id = self.request_id;
        // This can overflow, but that's fine for our use case
        self.request_id += 1;
        request_id
    }

    pub fn eval(&mut self, opts: EvalOpts, path: impl AsRef<str>) -> Result<Option<Value>, Error> {
        let request_id = self.gen_request_id();
        let builder = CreateEvaluatorRequest::builder()
            .request_id(request_id)
            .allowed_modules(opts.allowed_modules)
            .allowed_resources(opts.allowed_resources)
            .output_format(opts.output_format);
        let request = match opts.project {
            Some(project) => builder.project(project).build(),
            None => builder
                .module_paths(vec![path.as_ref().to_string()])
                .build(),
        };
        let mut response = self.proto.create_evaluator_request(request)?;

        if let Some(message) = response.error.take() {
            return Err(Error::Pkl(PklError::parse(message)));
        }

        if response.request_id != request_id {
            return Err(Error::InvalidRequestId {
                expected: request_id,
                actual: response.request_id,
            });
        }

        let mut response = self.proto.evaluate_request(
            EvaluateRequest::builder()
                .request_id(request_id)
                .evaluator_id(response.evaluator_id.unwrap_or_default())
                .module_uri(format!("file://{}", path.as_ref()))
                .expr("output.value".to_string())
                .build(),
        )?;

        if let Some(message) = response.error.take() {
            return Err(Error::Pkl(PklError::parse(message)));
        }

        match response.result {
            Some(mut result) => Ok(Some(Decoder::new(Cursor::new(&mut result)).decode()?)),
            None => Ok(None),
        }
    }
}
