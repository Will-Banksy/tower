use std::{collections::BTreeMap, ffi::{CStr, CString}, marker::PhantomData, mem};

use im::OrdMap;
use llvm_sys::{core::*, error_handling::{LLVMEnablePrettyStackTrace, LLVMInstallFatalErrorHandler}, execution_engine::{LLVMCreateExecutionEngineForModule, LLVMDisposeExecutionEngine, LLVMGetFunctionAddress, LLVMLinkInMCJIT, LLVMRunFunctionAsMain}, ir_reader::LLVMParseIRInContext, prelude::*, target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget}, LLVMBuilder, LLVMContext, LLVMModule};

use crate::analyser::{tree::{TypedTree, TypedTreeNode}, ttype::{OpaqueTypeKind, Type}};

const LLVM_ADDRESS_SPACE_GENERIC: u32 = 0;
const LLVM_FALSE: i32 = 0;
const LLVM_TRUE: i32 = 1;
const LLVM_STATUS_SUCCESS: i32 = 0;

macro_rules! cstr {
	($rust_str: literal) => {
		$rust_str.as_ptr() as *const i8
	};
}

macro_rules! cstrv {
	($rust_str: expr) => {
		CString::new($rust_str.as_bytes()).unwrap().into_bytes_with_nul().as_ptr() as *const i8
	};
}

pub struct CompiledProgram {
	context: *mut LLVMContext,
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
			LLVMShutdown();
		}
	}
}

struct CompileContext {
	pub context: LLVMContextRef,
	pub builder: LLVMBuilderRef,
	pub builtins: BTreeMap<String, LLVMValueRef>
}

impl CompileContext {
	extern "C" fn error_handler(reason: *const std::ffi::c_char) {
		let reason = unsafe {
			CStr::from_ptr(reason).to_str().unwrap()
		};
		eprintln!("LLVM Fatal Error: {reason}");
	}

	pub fn new() -> Self {
		let (context, builder) = unsafe {
			let context = LLVMContextCreate();
			let builder = LLVMCreateBuilderInContext(context);

			LLVMEnablePrettyStackTrace();
			LLVMInstallFatalErrorHandler(Some(CompileContext::error_handler));

			(context, builder)
		};

		CompileContext {
			context,
			builder,
			builtins: BTreeMap::new()
		}
	}

	pub fn add_builtin(&mut self) {
		let builtin = include_bytes!("../builtin.bc");
		let cbuiltin = builtin.as_ptr() as *const i8;

		unsafe {
			let buf = LLVMCreateMemoryBufferWithMemoryRange(cbuiltin, builtin.len(), cstr!("builtin_ir\0"), LLVM_TRUE);
			let mut builtin_module = LLVMModuleCreateWithNameInContext(cstr!("builtin\0"), self.context);
			let mut out_msg = mem::zeroed();

			if LLVMParseIRInContext(self.context, buf, &mut builtin_module, &mut out_msg) != 0 {
				eprintln!("Failed to add builtin module: {}", CStr::from_ptr(out_msg).to_str().unwrap());
			}

			// Collect all functions that are defined in the builtin module
			let mut f = LLVMGetFirstFunction(builtin_module);
			// let mut fbody = LLVMGetFirstBasicBlock(f); // Can check if this returns a nullptr or not to check whether the function has a body or not
			if f != std::ptr::null_mut() {
				let cfnname = LLVMGetValueName2(f, &mut 10);
				let fnname = CStr::from_ptr(cfnname).to_str().unwrap();
				if fnname.starts_with("__") { // Only want tower builtins - Which all start with __
					self.builtins.insert(fnname.to_string(), f);
					eprintln!("Collected {fnname} from builtin module");
				}
			}
			loop {
				f = LLVMGetNextFunction(f);
				if f != std::ptr::null_mut() {
					let cfnname = LLVMGetValueName2(f, &mut 10);
					let fnname = CStr::from_ptr(cfnname).to_str().unwrap();
					if fnname.starts_with("__") {
						self.builtins.insert(fnname.to_string(), f);
						eprintln!("Collected {fnname} from builtin module");
					}
				} else {
					break;
				}
			}
		}
	}

	pub fn create_module<'a>(&'a self, name: &str) -> ModuleContext<'a> {
		unsafe {
			let module = LLVMModuleCreateWithNameInContext(cstrv!(name), self.context);

			// Declare all builtin functions in the new module, and pass those function values to the ModuleContext // BUG: Causes crashes
			let mut builtins = BTreeMap::new();
			for (fnname, fnvalue) in &self.builtins {
				let fntype = LLVMGlobalGetValueType(*fnvalue);
				eprint!("\nForward declaring builtin fn {fnname} of LLVM type: ");
				LLVMDumpType(fntype);
				eprintln!();
				let modfnvalue = LLVMAddFunction(module, cstrv!(fnname), fntype);
				builtins.insert(fnname.clone(), modfnvalue);
			}

			ModuleContext {
				context: self.context,
				builder: self.builder,
				module,
				typedefs: BTreeMap::new(),
				functions: BTreeMap::new(),
				builtins,
				_lifetime: PhantomData
			}
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
	typedefs: BTreeMap<String, LLVMTypeRef>,
	functions: BTreeMap<String, (LLVMTypeRef, LLVMValueRef)>,
	builtins: BTreeMap<String, LLVMValueRef>,
	_lifetime: PhantomData<&'a ()>
}

impl<'a> ModuleContext<'a> {
	pub fn compile_module(&mut self, module: &TypedTreeNode) {
		if let TypedTree::Module { name: _, elems } = &module.tree {
			let mut functions_left: Vec<TypedTreeNode> = elems.iter().filter_map(|(_, enode)| if let TypedTree::Function { .. } = enode.tree { Some(enode.clone()) } else { None }).collect();

			while !functions_left.is_empty() {
				let mut to_remove = Vec::new();
				for (i, f) in functions_left.iter().enumerate() {
					if self.compile_function(f) {
						to_remove.push(i);
					}
				}
				assert_ne!(to_remove.len(), 0);
				for i in to_remove {
					functions_left.remove(i);
				}
			}
		}
	}

	/// Compiles and adds the passed in function to the module, returning true on success and false if other elements that are needed are not
	/// compiled yet
	pub fn compile_function(&mut self, func: &TypedTreeNode) -> bool {
		if let TypedTree::Function { name, effect, body } = &func.tree {
			eprintln!("Compiling function: {name}");

			for node in body {
				if let TypedTree::Word(name) = &node.tree {
					if !self.functions.contains_key(name) {
						return false;
					}
				}
			}

			let fntype = self.llvm_type(&Type::Function { name: name.to_string(), effect: effect.clone() });
			unsafe {
				let fnvalue = LLVMAddFunction(self.module, cstrv!(name), fntype);

				let block = LLVMAppendBasicBlockInContext(self.context, fnvalue, cstr!("entry\0"));
				LLVMPositionBuilderAtEnd(self.builder, block);

				let bppv = LLVMGetParam(fnvalue, 0);
				let sppv = LLVMGetParam(fnvalue, 1);
				let eppv = LLVMGetParam(fnvalue, 2);
				let mut bpv = LLVMBuildLoad2(self.builder, LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC), bppv, cstr!("bp\0"));
				let mut spv = LLVMBuildLoad2(self.builder, LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC), sppv, cstr!("sp\0"));
				let mut epv = LLVMBuildLoad2(self.builder, LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC), eppv, cstr!("ep\0"));

				for node in body {
					match &node.tree {
						TypedTree::Word(word) => {
							let (wordfn_type, wordfn) = self.functions.get(word).unwrap();

							let mut wordargs = [
								bppv,
								sppv,
								eppv
							];
							LLVMBuildCall2(self.builder, *wordfn_type, *wordfn, wordargs.as_mut_ptr(), 3, cstr!("\0"));
							bpv = LLVMBuildLoad2(self.builder, LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC), bppv, cstr!("bp\0"));
							spv = LLVMBuildLoad2(self.builder, LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC), sppv, cstr!("sp\0"));
							epv = LLVMBuildLoad2(self.builder, LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC), eppv, cstr!("ep\0"));
						},
						TypedTree::BuiltinWord(word) => {
							let wordfn = self.builtins.get(word).unwrap();
							let wordfn_type = LLVMGlobalGetValueType(*wordfn);

							let mut wordargs = [
								bppv,
								sppv,
								eppv
							];
							LLVMBuildCall2(self.builder, wordfn_type, *wordfn, wordargs.as_mut_ptr(), 3, cstr!("\0"));
							bpv = LLVMBuildLoad2(self.builder, LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC), bppv, cstr!("bp\0"));
							spv = LLVMBuildLoad2(self.builder, LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC), sppv, cstr!("sp\0"));
							epv = LLVMBuildLoad2(self.builder, LLVMPointerTypeInContext(self.context, LLVM_ADDRESS_SPACE_GENERIC), eppv, cstr!("ep\0"));
						},
						TypedTree::Literal { ty, value } => todo!(),
						TypedTree::Constructor { ty, effect } => todo!(),
						TypedTree::FieldAccess { name } => todo!(),
						_ => unreachable!()
					}
				}

				self.functions.insert(name.to_string(), (fntype, fnvalue));

				true
			}
		} else {
			unreachable!();
		}
	}

	/// Returns the LLVM LLVMTypeRef for the passed-in tower type
	pub fn llvm_type(&mut self, ty: &Type) -> LLVMTypeRef {
		let res: LLVMTypeRef = unsafe { match ty {
			Type::Opaque { size, kind } => {
				match kind {
					OpaqueTypeKind::Bool => LLVMInt1TypeInContext(self.context),
					OpaqueTypeKind::UnsignedInt | OpaqueTypeKind::SignedInt => {
						match size.unwrap() * 8 {
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
				if self.typedefs.contains_key(name) {
					return *self.typedefs.get(name).unwrap();
				}

				if !sum_type {
					fields.iter().for_each(|(fname, ftype)| { println!("inner ftype of {fname}: {}", ftype.name()); });
					let mut agg_elem_types: Vec<LLVMTypeRef> = fields.iter().map(|(_, ftype)| self.llvm_type(ftype)).collect();
					let agg_type = LLVMStructCreateNamed(self.context, cstrv!(name));
					LLVMStructSetBody(agg_type, agg_elem_types.as_mut_ptr(), agg_elem_types.len() as u32, LLVM_FALSE);
					// let agg_type = LLVMStructTypeInContext(self.context, agg_elem_types.as_mut_ptr(), agg_elem_types.len() as u32, LLVM_FALSE);
					self.typedefs.insert(name.to_string(), agg_type);
					agg_type
				} else {
					todo!()
				}
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
				LLVMFunctionType(LLVMVoidTypeInContext(self.context), args.as_mut_ptr(), 3, LLVM_FALSE)
			},
		}};

		res
	}

	fn dump(&mut self) {
		unsafe {
			LLVMDumpModule(self.module);
		}
	}
}

pub fn compile(typed_tree: TypedTreeNode) -> CompiledProgram {
	let mut context = CompileContext::new();

	context.add_builtin();

	let mut module = context.create_module("main");

	module.compile_module(&typed_tree);

	module.dump();

	todo!()

	// let mut typelist = Vec::new();

	// if let TypedTree::Module { name: _, elems } = typed_tree.tree {
	// 	for (ename, enode) in elems {
	// 		if let TypedTree::Type(ttype) = enode.tree {
	// 			eprintln!("Collecting LLVM type of {ename}");
	// 			typelist.push(module.llvm_type(&ttype));
	// 		}
	// 	}
	// }

	// unsafe {
	// 	let mut mainfn_params = [
	// 		LLVMInt32TypeInContext(context.context),
	// 		LLVMPointerTypeInContext(context.context, LLVM_ADDRESS_SPACE_GENERIC)
	// 	];
	// 	let mainfn_ty = LLVMFunctionType(LLVMInt32TypeInContext(context.context), mainfn_params.as_mut_ptr(), 2, LLVM_FALSE);
	// 	let mainfn = LLVMAddFunction(module.module, cstr!("main\0"), mainfn_ty);

	// 	let block = LLVMAppendBasicBlockInContext(context.context, mainfn, cstr!("entry\0"));
	// 	LLVMPositionBuilderAtEnd(module.builder, block);

	// 	for llvmtype in typelist {
	// 		let _ = LLVMBuildAlloca(module.builder, llvmtype, cstr!("alloca\0"));
	// 	}

	// 	LLVMDumpModule(module.module);
	// }
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
		let mainfn = LLVMAddFunction(module, cstr!("main\0"), mainfn_ty);

		let block = LLVMAppendBasicBlockInContext(ctx, mainfn, cstr!("entry\0"));
		LLVMPositionBuilderAtEnd(builder, block);

		let agg_x20_ty = LLVMArrayType2(agg_type, 20);
		let agg_alloca = LLVMBuildAlloca(builder, agg_x20_ty, cstr!("agg_alloca\0"));

		let argv_vals = LLVMGetParam(mainfn, 1);
		let mut gep_indices = [
			LLVMConstInt(LLVMInt32TypeInContext(ctx), 0, LLVM_FALSE),
		];
		let argv_first_ptr = LLVMBuildGEP2(builder, LLVMPointerTypeInContext(ctx, LLVM_ADDRESS_SPACE_GENERIC), argv_vals, gep_indices.as_mut_ptr(), 1, cstr!("argv_first_ptr\0"));
		let argv_first_val = LLVMBuildLoad2(builder, LLVMPointerTypeInContext(ctx, LLVM_ADDRESS_SPACE_GENERIC), argv_first_ptr, cstr!("argv_first\0"));
		let mut puts_args = [
			argv_first_val
		];
		let _puts_res = LLVMBuildCall2(builder, puts_ty, puts, puts_args.as_mut_ptr(), 1, cstr!("puts_ret\0"));

		let mut gep_indices = [
			LLVMConstInt(LLVMInt32TypeInContext(ctx), 1, LLVM_FALSE),
		];
		let argv_second_ptr = LLVMBuildGEP2(builder, LLVMPointerTypeInContext(ctx, LLVM_ADDRESS_SPACE_GENERIC), argv_vals, gep_indices.as_mut_ptr(), 1, cstr!("argv_second_ptr\0"));
		let argv_second_val = LLVMBuildLoad2(builder, LLVMPointerTypeInContext(ctx, LLVM_ADDRESS_SPACE_GENERIC), argv_second_ptr, cstr!("argv_second\0"));
		let mut printf_args = [
			argv_second_val
		];
		let _printf_res = LLVMBuildCall2(builder, printf_ty, printf, printf_args.as_mut_ptr(), 1, cstr!("printf_ret\0"));

		let lhs = LLVMConstInt(LLVMInt32TypeInContext(ctx), 42, LLVM_FALSE);
		let rhs = LLVMGetParam(mainfn, 0);
		let result = LLVMBuildAdd(builder, lhs, rhs, cstr!("result\0"));

		// LLVMParseIRInContext // TODO: Try using this to read an IR stub into a module?

		// LLVMAddGlobalMapping // Need to use this to link functions between modules. Need to declare the function in the module that is trying to call it first of course

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
			cstr!("Hello\0"), cstr!(" world\n\0")
		];
		let env = [
			cstr!("Environment?\0")
		];
		let ret = LLVMRunFunctionAsMain(ee, mainfn, 2, argv.as_ptr(), env.as_ptr());

		println!("Test function run by LLVM returned with: {ret}");

		LLVMDisposeExecutionEngine(ee);
		LLVMContextDispose(ctx);
		LLVMShutdown();
	}
}