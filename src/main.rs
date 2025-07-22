use crate::{
    client::{Project, Uri},
    evaluator::{EvalOpts, Evaluator},
    protocol::Protocol,
};

mod client;
mod decoder;
mod errors;
mod evaluator;
mod protocol;
mod server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protocol = Protocol::new()?;
    let mut evaluator = Evaluator::new(protocol);

    let mut opts = EvalOpts::default();
    opts.output_format = "yaml".to_string();
    opts.allowed_modules = vec![
        "pkl:".to_string(),
        "repl:text".to_string(),
        "projectpackage://pkg.pkl-lang.org/pkl-k8s/*".to_string(),
        "file://example/*".to_string(),
    ];
    opts.allowed_resources = vec![
        "prop:pkl.outputFormat".to_string(),
        "https://pkg.pkl-lang.org/pkl-k8s/k8s".to_string(),
        "https://github.com/apple/pkl-k8s/releases/download/k8s@1.0.1/k8s".to_string(),
        "file://example/input.json".to_string(),
    ];
    opts.project = Some(Project::from_path("example/")?);

    let value = evaluator.eval(&opts, Uri::File("example/app.pkl".into()))?;

    println!("{:#?}", value);

    Ok(())
}
