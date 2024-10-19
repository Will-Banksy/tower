#[derive(Debug)]
pub enum RuntimeError {
	StackUnderflowError,
	Utf8Error(Vec<u8>),
	FunctionMissingError(String),
	ModuleNotFoundError
}