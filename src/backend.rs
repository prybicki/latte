//use inkwell::{OptimizationLevel, AddressSpace};
//use inkwell::builder::Builder;
//use inkwell::context::Context;
//use inkwell::execution_engine::{ExecutionEngine, JitFunction};
//use inkwell::module::Module;
//use inkwell::module::Linkage;
//use inkwell::targets::{InitializationConfig, Target};
//use std::error::Error;
//
//use crate::ast;
//use crate::ast::Type;
//use inkwell::values::BasicValue;
//use inkwell::types::{AnyType, AnyTypeEnum};
//
//struct Backend<'llvm> {
//    llvm: &'llvm Context,
//    md: Module<'llvm>,
//    bd: Builder<'llvm>,
//}
//
//impl<'llvm> Backend<'llvm> {
//    fn new(llvm: &'llvm Context, mod_name: &str) -> Ctx<'llvm> {
//        let md = llvm.create_module(mod_name);
//        let bd = llvm.create_builder();
//        Ctx {llvm, md, bd}
//    }
//
//    fn get_llvm_type(&self, ttype: &ast::Type) -> AnyTypeEnum<'llvm> {
//        match ttype {
//            Type::Int => self.llvm.i32_type().as_any_type_enum(),
//            Type::Bool => self.llvm.bool_type().as_any_type_enum(),
//            Type::Str => self.llvm.i8_type().ptr_type(AddressSpace::Generic).as_any_type_enum(),
//            Type::Void => self.llvm.void_type().as_any_type_enum(),
//            Type::Invalid => panic!("cannot convert invalid ast type into llvm type"),
//        }
//    }
//
//    fn compile_fndef(&self, fndef: &ast::FnDef) {
//    //    let args_types = std::iter::repeat(ret_type)
//    //        .take(proto.args.len())
//    //        .map(|f| f.into())
//    //        .collect::<Vec<BasicTypeEnum>>();
//    //    let args_types = args_types.as_slice();
//    //
//        let (ret_type, args_types) = fndef.get_signature();
//        let arg_types = args_types.iter().map(|x| self.get_llvm_type(x)).collect();
//        let fn_type = match ret_type {
//            Type::Void => self.llvm.void_type().fn_type(arg_types),
//            Type::Invalid =>
//        }
//        let (ret_type, args_types) = (
//                BasicValue::try_from(self.get_llvm_type(&ret_type)),
//
//            );
//
//        let fn_type = ret_type.fn_type(args_types, false);
//        let fn_val = self.module.add_function(fndef.ident.as_ref(), fn_type, None);
//
//
//    //    let fn_val = self.module.add_function(proto.name.as_str(), fn_type, None);
//    //
//    //    // set arguments names
//    //    for (i, arg) in fn_val.get_param_iter().enumerate() {
//    //        arg.into_float_value().set_name(proto.args[i].as_str());
//    //    }
//    //
//    //    // finally return built prototype
//    //    Ok(fn_val)
//    //}
//    }
//
//    fn compile_prog(&self, prog: &ast::Program) {
//
//    }
//}
//
//pub fn compile(ast: &ast::Program) -> () {
//
//    let llvm = Context::create();
//    let ctx = Ctx::new(&llvm, "simplest");
//    return compile_prog()
//}
//
////impl<'llvm> Ctx<'llvm> {
////    fn new(llvm: &'ctx Context, md: &'a Module<'ctx>, bd: &'a, Builder<'ctx>) {
////        Ctx
////    }
//
//
////}
//
/////// Compiles the specified `Prototype` into an extern LLVM `FunctionValue`.
////fn compile_prototype(&self, proto: &Prototype) -> Result<FunctionValue<'ctx>, &'static str> {
////
////    let args_types = std::iter::repeat(ret_type)
////        .take(proto.args.len())
////        .map(|f| f.into())
////        .collect::<Vec<BasicTypeEnum>>();
////    let args_types = args_types.as_slice();
////
////    let fn_type = self.context.f64_type().fn_type(args_types, false);
////    let fn_val = self.module.add_function(proto.name.as_str(), fn_type, None);
////
////    // set arguments names
////    for (i, arg) in fn_val.get_param_iter().enumerate() {
////        arg.into_float_value().set_name(proto.args[i].as_str());
////    }
////
////    // finally return built prototype
////    Ok(fn_val)
////}
//
//
////fn compile_fn(&mut self, fndef: &ast::FnDef) {
////    let ret_type = self.context.f64_type();
////    let proto = &self.function.prototype;
////    let function = self.compile_prototype(proto)?;
////
////    // got external function, returning only compiled prototype
////    if self.function.body.is_none() {
////        return Ok(function);
////    }
////
////    let entry = self.context.append_basic_block(function, "entry");
////
////    self.builder.position_at_end(&entry);
////
////    // update fn field
////    self.fn_value_opt = Some(function);
////
////    // build variables map
////    self.variables.reserve(proto.args.len());
////
////    for (i, arg) in function.get_param_iter().enumerate() {
////        let arg_name = proto.args[i].as_str();
////        let alloca = self.create_entry_block_alloca(arg_name);
////
////        self.builder.build_store(alloca, arg);
////
////        self.variables.insert(proto.args[i].clone(), alloca);
////    }
////
////    // compile body
////    let body = self.compile_expr(self.function.body.as_ref().unwrap())?;
////
////    self.builder.build_return(Some(&body));
////
////    // return the whole thing after verification and optimization
////    if function.verify(true) {
////        self.fpm.run_on(&function);
////
////        Ok(function)
////    } else {
////        unsafe {
////            function.delete();
////        }
////
////        Err("Invalid generated function.")
////    }
////}
//
////       fn print_int(&self) -> String {
////        let i32 = self.context.i32_type();
////        let i8pp = self.context.i8_type().ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Generic);
////        self.module.add_function("main", self.context.i32_type().fn_type(&[i32.into(), i8pp.into(), i8pp.into()], false), Some(Linkage::Internal));
////
////        return self.module.print_to_string().to_string();
////    }
////}
//
//
