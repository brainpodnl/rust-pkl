use std::io::Cursor;

use tracing::instrument;

use crate::{
    client::{CreateEvaluatorRequest, EvaluateRequest, Project, Uri},
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

    #[instrument(skip(self, opts))]
    pub fn eval(&mut self, opts: &EvalOpts, uri: Uri) -> Result<Option<Value>, Error> {
        let request_id = self.gen_request_id();
        let module_paths = [uri.to_string()];

        let mut request = CreateEvaluatorRequest::default();
        request.request_id = request_id;
        request.allowed_modules = Some(&opts.allowed_modules);
        request.allowed_resources = Some(&opts.allowed_resources);
        request.output_format = Some(&opts.output_format);

        if opts.project.is_some() {
            request.project = opts.project.as_ref();
        } else {
            request.module_paths = Some(&module_paths);
        }

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

        let mut request = EvaluateRequest::default();
        request.request_id = request_id;
        request.evaluator_id = response.evaluator_id.unwrap_or_default();
        request.module_uri = uri;
        request.expr = Some("output.value");

        let mut response = self.proto.evaluate_request(request)?;

        if let Some(message) = response.error.take() {
            return Err(Error::Pkl(PklError::parse(message)));
        }

        match response.result {
            Some(mut result) => {
                let mut decoder = Decoder::new(Cursor::new(&mut result));
                Ok(Some(decoder.decode()?))
            }
            None => Ok(None),
        }
    }
}
