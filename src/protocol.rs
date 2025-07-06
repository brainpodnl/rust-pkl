use std::{
    io::Write,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use rmp_serde::{Serializer, config::BytesMode};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    client::{CreateEvaluatorRequest, EvaluateRequest},
    decoder::Decoder,
    errors::Error,
    server::{CreateEvaluatorResponse, EvaluateResponse, Response},
};

pub trait Message {
    const CODE: u64;
}

pub struct Protocol {
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
}

impl Protocol {
    pub fn new() -> Result<Self, Error> {
        let mut child = Command::new("pkl")
            .arg("server")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child.stdin.take().ok_or(Error::Pipe)?;
        let stdout = child.stdout.take().ok_or(Error::Pipe)?;

        Ok(Self {
            child,
            stdin,
            stdout,
        })
    }

    pub fn create_evaluator_request(
        &mut self,
        request: CreateEvaluatorRequest,
    ) -> Result<CreateEvaluatorResponse, Error> {
        self.send(request)?;
        Ok(self.recv()?)
    }

    pub fn evaluate_request(
        &mut self,
        request: EvaluateRequest,
    ) -> Result<EvaluateResponse, Error> {
        self.send(request)?;
        Ok(self.recv()?)
    }

    fn recv<T>(&mut self) -> Result<T, Error>
    where
        T: Message + DeserializeOwned,
        T: TryFrom<Response, Error = Error>,
    {
        Decoder::new(&mut self.stdout).decode_response_typed::<T>()
    }

    fn send<M: Message + Serialize>(&mut self, message: M) -> Result<(), Error> {
        let mut serializer = Serializer::new(&mut self.stdin)
            .with_struct_map()
            .with_bytes(BytesMode::ForceAll);

        (M::CODE, message).serialize(&mut serializer)?;
        self.stdin.flush()?;

        Ok(())
    }

    async fn close(mut self) -> Result<(), Error> {
        let _ = self.child.kill();
        Ok(())
    }
}
