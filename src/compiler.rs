use std::{ffi::CStr, marker::PhantomData, mem};

use llvm_sys::{core::*, execution_engine::{LLVMCreateExecutionEngineForModule, LLVMDisposeExecutionEngine, LLVMGetFunctionAddress, LLVMLinkInMCJIT, LLVMRunFunctionAsMain}, prelude::*, target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget}, LLVMBuilder, LLVMContext, LLVMModule};

use crate::analyser::{tree::{TypedTree, TypedTreeNode}, ttype::{OpaqueTypeKind, Type}};

const LLVM_ADDRESS_SPACE_GENERIC: u32 = 0;
const LLVM_FALSE: i32 = 0;
const LLVM_TRUE: i32 = 1;
const LLVM_STATUS_SUCCESS: i32 = 0;

macro_rules! cstr {
	($rust_str: expr) => {
		$rust_str.as_ptr() as *const i8
	};
}

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
	pub context: *mut LLVMContext,
	pub builder: *mut LLVMBuilder
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

	pub fn create_module<'a>(&'a self, name: &str) -> ModuleContext<'a> {
		let module = unsafe {
			LLVMModuleCreateWithNameInContext(cstr!(name), self.context)
		};
		ModuleContext {
			context: self.context,
			builder: self.builder,
			module,
			_lifetime: PhantomData
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

struct ModuleContext<'a> {
	context: *mut LLVMContext,
	builder: *mut LLVMBuilder,
	module: *mut LLVMModule,
	_lifetime: PhantomData<&'a ()>
}

impl<'a> ModuleContext<'a> {
	pub fn compile_function(&mut self, func: &TypedTreeNode) {
		if let TypedTree::Function { name, effect, body } = &func.tree {
			todo!()
		} else {
			unreachable!();
		}
	}

	pub fn compile_typedef(&mut self, typ: &TypedTreeNode) {
		if let TypedTree::Type(type_info) = &typ.tree {
			if let Type::Transparent { name, fields, sum_type } = type_info {
				if !sum_type {
					unsafe {
						// let elem_types =
						// LLVMStructTypeInContext(self.context, ElementTypes, ElementCount, Packed)
						// TODO: Probably can use add alias to name a struct type?
						// LLVMAddAlias2(self., ValueTy, AddrSpace, Aliasee, Name)
					}
					todo!()
				} else {
					todo!()
				}
			} else {
				unreachable!()
			}
		} else {
			unreachable!();
		}
	}

	/// Returns the LLVM LLVMTypeRef for the passed-in tower type
	pub fn llvm_type(&mut self, ty: Type) -> LLVMTypeRef {
		let res: LLVMTypeRef = unsafe { match ty {
			Type::Opaque { size, kind } => {
				match kind {
					OpaqueTypeKind::Bool => LLVMInt1TypeInContext(self.context),
					OpaqueTypeKind::UnsignedInt | OpaqueTypeKind::SignedInt => {
						match size.unwrap() {
							8 => LLVMInt8TypeInContext(self.context),
							16 => LLVMInt16TypeInContext(self.context),
							32 => LLVMInt32TypeInContext(self.context),
							64 => LLVMInt64TypeInContext(self.context),
							128 => LLVMInt128TypeInContext(self.context),
							_ => unreachable!()
						}
					},
					OpaqueTypeKind::Float => {
						match size.unwrap() {
							32 => LLVMFloatTypeInContext(self.context),
							64 => LLVMDoubleTypeInContext(self.context),
							_ => unreachable!()
						}
					},
					OpaqueTypeKind::Str => {
						LLVMArrayType2(
							LLVMInt8TypeInContext(self.context),
							size.unwrap() as u64
						)
					},
					OpaqueTypeKind::Array => todo!(),
				}
			},
			Type::Transparent { name, fields, sum_type } => {
				todo!() // TODO
				// LLVMStructTypeInContext
				// Unions?
			},
			Type::Reference { to: _ } => LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC),
			Type::Generic { name: _ } => unreachable!(),
			Type::Function { name: _, effect: _ } => {
				// NOTE: I don't think instructions will be processed here
				// Tower functions always take: i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr
				// i.e., 3 pointers
				let mut args = [
					LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC),
					LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC),
					LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC)
				];
				LLVMFunctionType(LLVMVoidTypeInContext(self.context), args.as_mut_ptr(), 3, LLVM_TRUE)
			},
		}};

		res
	}
}

fn compile(typed_tree: TypedTreeNode) -> CompiledProgram {
	let context = CompileContext::new();

	match typed_tree.tree {
		TypedTree::Module { name, elems } => todo!(),
		TypedTree::Function { name, effect, body } => todo!(),
		TypedTree::Type(_) => todo!(),
		TypedTree::Word(_) => todo!(),
		TypedTree::BuiltinWord(_) => todo!(),
		TypedTree::Literal { ty, value } => todo!(),
		TypedTree::Constructor { ty, effect } => todo!(),
		TypedTree::FieldAccess { name } => todo!(),
	}

	todo!()
}

pub fn compile_test_program() {
	unsafe {
		let ctx = LLVMContextCreate();
		let builder = LLVMCreateBuilderInContext(ctx);

		let module = LLVMModuleCreateWithNameInContext("main\0".as_ptr() as *const i8, ctx);

		let mut puts_params = [
			LLVMPointerTypeInContext(ctx, LLVM_ADDRESS_SPACE_GENERIC)
		];
		let puts_ty = LLVMFunctionType(LLVMInt32TypeInContext(ctx), puts_params.as_mut_ptr(), 1, LLVM_FALSE);
		let puts = LLVMAddFunction(module, "puts\0".as_ptr() as *const i8, puts_ty);

		let mut printf_params = [
			LLVMPointerTypeInContext(ctx, LLVM_ADDRESS_SPACE_GENERIC)
		];
		let printf_ty = LLVMFunctionType(LLVMInt32TypeInContext(ctx), printf_params.as_mut_ptr(), 1, LLVM_TRUE);
		let printf = LLVMAddFunction(module, cstr!("printf\0"), printf_ty);

		let some_str = LLVMAddGlobal(module, LLVMArrayType2(LLVMInt8TypeInContext(ctx), 8), "some_str\0".as_ptr() as *const i8);
		LLVMSetGlobalConstant(some_str, LLVM_TRUE);
		LLVMSetInitializer(some_str, LLVMConstStringInContext(ctx, "somestr".as_ptr() as *const i8, 7, LLVM_FALSE));

		// let agg_type = LLVMStructTypeInContext(ctx, agg_elem_types.as_mut_ptr(), 3, LLVM_TRUE);
		// LLVMAddAlias2(module, agg_type, LLVM_ADDRESS_SPACE_GENERIC, Aliasee, Name)
		// let agg_val = LLVMAddGlobal(module, agg_type, cstr!("agg\0"));
		// LLVMAddAlias2(module, agg_type, LLVM_ADDRESS_SPACE_GENERIC, agg_val, cstr!("agg\0"));
		let mut agg_elem_types = [
			LLVMInt32TypeInContext(ctx),
			LLVMInt16TypeInContext(ctx),
			LLVMInt16TypeInContext(ctx),
		];
		let agg_type = LLVMStructCreateNamed(ctx, cstr!("agg\0"));
		LLVMStructSetBody(agg_type, agg_elem_types.as_mut_ptr(), 3, LLVM_FALSE);

		let mut mainfn_params = [
			LLVMInt32TypeInContext(ctx),
			LLVMPointerTypeInContext(ctx, LLVM_ADDRESS_SPACE_GENERIC)
		];
		let mainfn_ty = LLVMFunctionType(LLVMInt32TypeInContext(ctx), mainfn_params.as_mut_ptr(), 2, LLVM_FALSE);
		let mainfn = LLVMAddFunction(module, "main\0".as_ptr() as *const i8, mainfn_ty);

		let block = LLVMAppendBasicBlockInContext(ctx, mainfn, "entry\0".as_ptr() as *const i8);
		LLVMPositionBuilderAtEnd(builder, block);

		let agg_x20_ty = LLVMArrayType2(agg_type, 20);
		let agg_alloca = LLVMBuildAlloca(builder, agg_x20_ty, cstr!("agg_alloca\0"));

		let argv_vals = LLVMGetParam(mainfn, 1);
		let mut gep_indices = [
			LLVMConstInt(LLVMInt32TypeInContext(ctx), 0, LLVM_FALSE),
		];
		let argv_first_ptr = LLVMBuildGEP2(builder, LLVMPointerTypeInContext(ctx, LLVM_ADDRESS_SPACE_GENERIC), argv_vals, gep_indices.as_mut_ptr(), 1, "argv_first_ptr\0".as_ptr() as *const i8);
		let argv_first_val = LLVMBuildLoad2(builder, LLVMPointerTypeInContext(ctx, LLVM_ADDRESS_SPACE_GENERIC), argv_first_ptr, "argv_first\0".as_ptr() as *const i8);
		let mut puts_args = [
			argv_first_val
		];
		let _puts_res = LLVMBuildCall2(builder, puts_ty, puts, puts_args.as_mut_ptr(), 1, "puts_ret\0".as_ptr() as *const i8);

		let mut gep_indices = [
			LLVMConstInt(LLVMInt32TypeInContext(ctx), 1, LLVM_FALSE),
		];
		let argv_second_ptr = LLVMBuildGEP2(builder, LLVMPointerTypeInContext(ctx, LLVM_ADDRESS_SPACE_GENERIC), argv_vals, gep_indices.as_mut_ptr(), 1, "argv_second_ptr\0".as_ptr() as *const i8);
		let argv_second_val = LLVMBuildLoad2(builder, LLVMPointerTypeInContext(ctx, LLVM_ADDRESS_SPACE_GENERIC), argv_second_ptr, "argv_second\0".as_ptr() as *const i8);
		let mut printf_args = [
			argv_second_val
		];
		let _printf_res = LLVMBuildCall2(builder, printf_ty, printf, printf_args.as_mut_ptr(), 1, "printf_ret\0".as_ptr() as *const i8);

		let lhs = LLVMConstInt(LLVMInt32TypeInContext(ctx), 42, LLVM_FALSE);
		let rhs = LLVMGetParam(mainfn, 0);
		let result = LLVMBuildAdd(builder, lhs, rhs, "result\0".as_ptr() as *const i8);

		LLVMBuildRet(builder, result);

		LLVMDumpModule(module);

		// Setup for execution engine, then build execution engine
		LLVMLinkInMCJIT();
		LLVM_InitializeNativeTarget();
		LLVM_InitializeNativeAsmPrinter();

		let ee = {
			let mut ee = mem::MaybeUninit::uninit();
			let mut err = mem::zeroed();

			if LLVMCreateExecutionEngineForModule(ee.as_mut_ptr(), module, &mut err) != LLVM_STATUS_SUCCESS {
				println!("Execution engine creation failed: {:?}", CStr::from_ptr(err));
			}

			ee.assume_init()
		};

		let argv = [
			"Hello\0".as_ptr() as *const i8, " world\n\0".as_ptr() as *const i8
		];
		let env = [
			"Environment?\0".as_ptr() as *const i8
		];
		let ret = LLVMRunFunctionAsMain(ee, mainfn, 2, argv.as_ptr(), env.as_ptr());

		println!("Test function run by LLVM returned with: {ret}");

		LLVMDisposeExecutionEngine(ee);
		LLVMContextDispose(ctx);
		LLVMShutdown();
	}
}