use inkwell::{OptimizationLevel, AddressSpace};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::module::Linkage;
use inkwell::targets::{InitializationConfig, Target};
use std::error::Error;

use crate::ast;
use crate::ast::Type;
use inkwell::values::BasicValue;
use inkwell::types::{AnyType, AnyTypeEnum, BasicType, BasicTypeEnum};

struct Backend<'llvm> {
    llvm: &'llvm Context,
    md: Module<'llvm>,
    bd: Builder<'llvm>,
}

impl<'llvm> Backend<'llvm> {
    fn new(llvm: &'llvm Context, mod_name: &str) -> Backend<'llvm> {
        let md = llvm.create_module(mod_name);
        let bd = llvm.create_builder();
        Backend {llvm, md, bd}
    }

    fn get_llvm_basic_type(&self, ttype: &ast::Type) -> Option<BasicTypeEnum<'llvm>> {
        match ttype {
            Type::Int =>  Some(self.llvm.i32_type().as_basic_type_enum()),
            Type::Bool => Some(self.llvm.bool_type().as_basic_type_enum()),
            Type::Str =>  Some(self.llvm.i8_type().ptr_type(AddressSpace::Generic).as_basic_type_enum()),
            Type::Void => None,
        }
    }

    fn compile_fndef(&self, fndef: &ast::FnDef) {
    //    let args_types = std::iter::repeat(ret_type)
    //        .take(proto.args.len())
    //        .map(|f| f.into())
    //        .collect::<Vec<BasicTypeEnum>>();
    //    let args_types = args_types.as_slice();
    //
        let (ret_type, args_types) = fndef.get_signature();
        let arg_types: Vec<BasicTypeEnum> = args_types.iter().map(|x| self.get_llvm_basic_type(x).unwrap()).collect();
        let fn_type = match ret_type {
            Type::Void => self.llvm.void_type().fn_type(&arg_types, false),
            _ => self.get_llvm_basic_type(&ret_type).unwrap().fn_type(&arg_types, false),
        };

        let fn_val = self.md.add_function(fndef.ident.as_ref(), fn_type, None);
        let entry = self.llvm.append_basic_block(fn_val, "entry");
        self.bd.position_at_end(&entry);
        self.bd.build_return(Some(&self.llvm.i32_type().const_int(0, false)));

//        for (i, arg) in function.get_param_iter().enumerate() {
//            let arg_name = proto.args[i].as_str();
//            let alloca = self.create_entry_block_alloca(arg_name);
//
//            self.builder.build_store(alloca, arg);
//
//            self.variables.insert(proto.args[i].clone(), alloca);
//    }

        // compile body



    }

    fn compile_prog(&self, prog: &ast::Program) {
        for fndef in &prog.functions {
            self.compile_fndef(fndef);
        }
    }
}

pub fn compile(prog: &ast::Program) -> () {
    let llvm = Context::create();
    let backend = Backend::new(&llvm, "simplest");
    backend.compile_prog(prog);
    println!("KOD:\n{}", backend.md.print_to_string().to_string());
}

//impl<'llvm> Ctx<'llvm> {
//    fn new(llvm: &'ctx Context, md: &'a Module<'ctx>, bd: &'a, Builder<'ctx>) {
//        Ctx
//    }


//fn compile_fn(&mut self, fndef: &ast::FnDef) {
//    let ret_type = self.context.f64_type();
//    let proto = &self.function.prototype;
//    let function = self.compile_prototype(proto)?;
//
//    // got external function, returning only compiled prototype
//    if self.function.body.is_none() {
//        return Ok(function);
//    }
//
//
//    }
//}

//       fn print_int(&self) -> String {
//        let i32 = self.context.i32_type();
//        let i8pp = self.context.i8_type().ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Generic);
//        self.module.add_function("main", self.context.i32_type().fn_type(&[i32.into(), i8pp.into(), i8pp.into()], false), Some(Linkage::Internal));
//
//        return self.module.print_to_string().to_string();
//    }
//}


