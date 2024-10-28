use llvm_sys::{core::{LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext, LLVMDisposeBuilder}, LLVMBuilder, LLVMContext};

use crate::analyser::tree::TypedTreeNode;

struct CompiledProgram {
	context: *mut LLVMContext
}

impl CompiledProgram {
	pub fn new(context: *mut LLVMContext) -> Self {
		CompiledProgram {
			context
		}
	}
}

impl Drop for CompiledProgram {
	fn drop(&mut self) {
		unsafe {
			LLVMContextDispose(self.context);
		}
	}
}

struct CompileContext {
	context: *mut LLVMContext,
	builder: *mut LLVMBuilder
}

impl CompileContext {
	pub fn new() -> Self {
		let (context, builder) = unsafe {
			let context = LLVMContextCreate();
			let builder = LLVMCreateBuilderInContext(context);
			(context, builder)
		};

		CompileContext {
			context,
			builder
		}
	}
}

impl Drop for CompileContext {
	fn drop(&mut self) {
		unsafe {
			LLVMDisposeBuilder(self.builder);
		}
	}
}

fn compile(typed_tree: TypedTreeNode) -> CompiledProgram {
	let context = CompileContext::new();

	todo!()
}