use inkwell::{OptimizationLevel, AddressSpace};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::module::Linkage;
use inkwell::targets::{InitializationConfig, Target};
use std::error::Error;
use either::Either;

use crate::ast::{Type, Stmt, Exp, ExpNode, Ident, FnDef, StmtNode, Program};
use inkwell::values::{BasicValue, AnyValueEnum, BasicValueEnum, FunctionValue, AnyValue, CallSiteValue};
use inkwell::types::{AnyType, AnyTypeEnum, BasicType, BasicTypeEnum};
use std::convert::TryInto;
use std::collections::HashMap;

type FEnv<'llvm> = HashMap<Ident, FunctionValue<'llvm>>;


struct Backend<'llvm> {
    llvm: &'llvm Context,
    md: Module<'llvm>,
    bd: Builder<'llvm>,
    fenv: FEnv<'llvm>,
}

impl<'llvm> Backend<'llvm> {
    fn new(llvm: &'llvm Context, mod_name: &str) -> Backend<'llvm> {
        let md = llvm.create_module(mod_name);
        let bd = llvm.create_builder();
        let fenv = FEnv::new();
        Backend {llvm, md, bd, fenv}
    }

    fn get_llvm_basic_type(&self, ttype: &Type) -> Option<BasicTypeEnum<'llvm>> {
        match ttype {
            Type::Int =>  Some(self.llvm.i32_type().as_basic_type_enum()),
            Type::Bool => Some(self.llvm.bool_type().as_basic_type_enum()),
            Type::Str =>  Some(self.llvm.i8_type().ptr_type(AddressSpace::Generic).as_basic_type_enum()),
            Type::Void => None,
        }
    }

    fn compile_exp(&self, node: &ExpNode) -> Option<BasicValueEnum<'llvm>> {
        match &node.exp {
            Exp::Call(ident, args) => {
                let fnval = *self.fenv.get(ident).unwrap();
                let argsvals: Vec<BasicValueEnum> = args.iter().map(|x| self.compile_exp(x).unwrap()).collect();
                let res = self.bd.build_call(fnval, &argsvals, "").try_as_basic_value();
                match res {
                    Either::Left(l) => Some(l),
                    Either::Right(r) => unimplemented!()
                }
            },
            Exp::Int(val) => Some(self.llvm.i32_type().const_int(*val as u64, false).into()),
            _ => unimplemented!()
        }
    }

    fn compile_stmt(&self, node: &StmtNode) {
        match &node.stmt {
            Stmt::EStmt(exp_node) => {

            },
            Stmt::Ret(node) => {
                self.bd.build_return(Some(&self.compile_exp(node).unwrap()));
            }

            _ => unimplemented!()
        }
    }

    fn compile_fndef(&self, fndef: &FnDef) {
        let fnval = *self.fenv.get(&fndef.ident).unwrap();
        let entry = self.llvm.append_basic_block(fnval, "entry");
        self.bd.position_at_end(&entry);
        if let Stmt::BStmt(stmts) = &fndef.body.stmt {
            self.compile_stmt(stmts.first().unwrap())
        }
    }

    fn compile_prog(&mut self, prog: &Program) {
        for fndef in &prog.functions {
            let (ret_type, args_types) = fndef.get_signature();
            let arg_types: Vec<BasicTypeEnum> = args_types.iter().map(|x| self.get_llvm_basic_type(x).unwrap()).collect();
            let fn_type = match ret_type {
                Type::Void => self.llvm.void_type().fn_type(&arg_types, false),
                _ => self.get_llvm_basic_type(&ret_type).unwrap().fn_type(&arg_types, false),
            };
            let fnval = self.md.add_function(fndef.ident.as_ref(), fn_type, None);
            self.fenv.insert(fndef.ident.clone(), fnval);
        }

        for fndef in &prog.functions {
            self.compile_fndef(fndef);
        }
    }
}

pub fn compile(prog: &Program) -> () {
    let llvm = Context::create();
    let mut backend = Backend::new(&llvm, "simplest");
    backend.compile_prog(prog);
    println!("KOD:\n{}", backend.md.print_to_string().to_string());
}
