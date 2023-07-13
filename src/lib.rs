use std::process::ExitCode;
use thiserror::Error;
use num_bigint::BigUint;
use num_complex::Complex64;
use qsc::interpret::output::Receiver;
use qsc::interpret::{stateless, output, Value};
use qsc::{PackageStore, SourceMap, hir::PackageId};
use qsc::compile::{compile};

#[derive(Error, Debug)]
pub enum QsError {
    #[error("Error with message: `{error_text}`")]
    ErrorMessage { error_text: String }
}

fn compile_qs(source: &str) -> () {
    let store = PackageStore::new(qsc::compile::core());
    let dependencies: Vec<PackageId> = Vec::new();

    let sources = SourceMap::new(vec![("temp.qs".into(), source.into())], Some("".into()));
    let (unit, errors) = compile(&store, &dependencies, sources);

    if errors.is_empty() {
        ()
    } else {
        for error in errors {
            if let Some(source) = unit.sources.find_by_diagnostic(&error) {
                eprintln!("{:?}", source.clone());
            } else {
                eprintln!("{:?}", error);
            }
        }

        ()
    }
}

fn run_qs(source: &str) -> Result<ExecutionState, QsError> {
    let source_map = SourceMap::new(vec![("temp.qs".into(), source.into())], Some("".into()));

    let context = match stateless::Context::new(true, source_map) {
        Ok(context) => context,
        Err(errors) => {
            for error in errors {
                eprintln!("error: {:?}", error);
            }
            return Err(QsError::ErrorMessage { error_text: "context error".to_string() });
        }
    };
    let mut rec = ExecutionState::default();
    let result = context.eval(&mut rec);
    match result {
        Ok(value) => {
            println!("{value}");
            return Ok(rec);
        }
        Err(errors) => {
            for error in errors {
                if let Some(stack_trace) = error.stack_trace() {
                    eprintln!("{stack_trace}");
                }
                eprintln!("error: {error:?}");
            }
            return Err(QsError::ErrorMessage { error_text: "execution error".to_string() });
        }
    }
}

struct QubitState {
    id: String,
    amplitude_real: f64,
    amplitude_imaginary: f64,
}

struct ExecutionState {
    states: Vec<QubitState>,
    qubit_count: usize,
    messages: Vec<String>,
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self {
            states: Vec::new(),
            qubit_count: 0,
            messages: Vec::new(),
        }
    }
}

impl Receiver for ExecutionState {
    fn state(
        &mut self,
        states: Vec<(BigUint, Complex64)>,
        qubit_count: usize,
    ) -> Result<(), output::Error> {
        self.qubit_count = qubit_count;
        self.states = states.iter().map(|(qubit, amplitude)| {
            QubitState {
                id: output::format_state_id(&qubit, qubit_count),
                amplitude_real: amplitude.re,
                amplitude_imaginary: amplitude.im,
            }
        }).collect();

        Ok(())
    }

    fn message(&mut self, msg: &str) -> Result<(), output::Error> {
        self.messages.push(msg.to_string());
        Ok(())
    }
}

#[test]
fn test_run() {
    let source = "
    namespace MyQuantumApp {
        @EntryPoint()
        operation Main() : Unit {
            Message(\"Hello\");
        }
    }";
    let result = run_qs(source).unwrap();

    assert_eq!(result.messages.len(), 1);
    assert_eq!(result.messages[0], "Hello");

    assert_eq!(result.qubit_count, 0);
    assert_eq!(result.states.len(), 0);
}
