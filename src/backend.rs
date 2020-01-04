use crate::ast::*;
use inkwell::*;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::*;
use inkwell::types::*;
use either::Either;
use std::collections::HashMap;
use inkwell::memory_buffer::MemoryBuffer;
use std::path::Path;
use std::fs;
use crate::scoped_map::ScopedMap;

type FEnv<'llvm> = ScopedMap<Ident, FunctionValue<'llvm>>;
type VEnv<'llvm> = HashMap<Ident, BasicValueEnum<'llvm>>;

struct Backend<'llvm> {
    llvm: &'llvm Context,
    md: Module<'llvm>,
    bd: Builder<'llvm>,
    fenv: FEnv<'llvm>,
    venv: VEnv<'llvm>,
}

impl<'llvm> Backend<'llvm> {
    fn new(llvm: &'llvm Context, mod_name: &str) -> Backend<'llvm> {
        let md = llvm.create_module(mod_name);
        let bd = llvm.create_builder();
        let fenv = FEnv::new();
        let venv = VEnv::new();
        Backend {llvm, md, bd, fenv, venv}
    }

    fn get_llvm_basic_type(&self, ttype: &Type) -> Option<BasicTypeEnum<'llvm>> {
        match ttype {
            Type::Int =>  Some(self.llvm.i32_type().as_basic_type_enum()),
            Type::Bool => Some(self.llvm.bool_type().as_basic_type_enum()),
            Type::Str =>  Some(self.llvm.i8_type().ptr_type(AddressSpace::Generic).as_basic_type_enum()),
            Type::Void => None,
        }
    }

    fn get_llvm_default_value(&self, ttype: &Type) -> Option<BasicValueEnum<'llvm>> {
        match ttype {
            Type::Int => Some(self.llvm.i32_type().const_zero().into()),
            Type::Bool => Some(self.llvm.bool_type().const_zero().into()),
            Type::Str => unimplemented!(),
            Type::Void => None
        }
    }

    fn compile_bin_exp(&self, op: &BinaryOp, lexp: &ExpNode, rexp: &ExpNode) -> BasicValueEnum<'llvm> {
        let lval = self.compile_exp(lexp).unwrap().into_int_value();
        let rval = self.compile_exp(rexp).unwrap().into_int_value();
        let ltv = lexp.typeval.as_ref().unwrap();
        let rtv = rexp.typeval.as_ref().unwrap();
        match (op, ltv, rtv) {
//        (BinaryOp::Eq,  ExpTypeVal::Str(Some(l)), ExpTypeVal::Str(Some(r))) => ExpTypeVal::Bool(Some(*l == *r)),
//        (BinaryOp::Neq,  ExpTypeVal::Str(Some(l)), ExpTypeVal::Str(Some(r))) => ExpTypeVal::Bool(Some(*l != *r)),
            (BinaryOp::Eq, ExpTypeVal::Bool(_), ExpTypeVal::Bool(_)) => {
                self.bd.build_int_compare(IntPredicate::EQ, lval, rval, "bool_eq").into()
            },
            (BinaryOp::Eq, ExpTypeVal::Int(_), ExpTypeVal::Int(_)) => {
                self.bd.build_int_compare(IntPredicate::EQ, lval, rval, "int_eq").into()
            }
            (BinaryOp::Neq, ExpTypeVal::Bool(_), ExpTypeVal::Bool(_)) => {
                self.bd.build_int_compare(IntPredicate::NE, lval, rval, "bool_neq").into()
            }
            (BinaryOp::Neq, ExpTypeVal::Int(_), ExpTypeVal::Int(_)) => {
                self.bd.build_int_compare(IntPredicate::NE, lval, rval, "int_neq").into()
            }
            (BinaryOp::Or, ExpTypeVal::Bool(_), ExpTypeVal::Bool(_)) => {
                self.bd.build_or(lval, rval, "or").into()
            }
            (BinaryOp::And, ExpTypeVal::Bool(_), ExpTypeVal::Bool(_)) => {
                self.bd.build_and(lval, rval, "and").into()
            }
            (BinaryOp::Gt, ExpTypeVal::Int(_), ExpTypeVal::Int(_)) => {
                self.bd.build_int_compare(IntPredicate::SGT, lval, rval, "gt").into()
            }
            (BinaryOp::Gte, ExpTypeVal::Int(_), ExpTypeVal::Int(_)) => {
                self.bd.build_int_compare(IntPredicate::SGE, lval, rval, "gte").into()
            }
            (BinaryOp::Lt, ExpTypeVal::Int(_), ExpTypeVal::Int(_)) => {
                self.bd.build_int_compare(IntPredicate::SLT, lval, rval, "lt").into()
            }
            (BinaryOp::Lte, ExpTypeVal::Int(_), ExpTypeVal::Int(_)) => {
                self.bd.build_int_compare(IntPredicate::SLE, lval, rval, "lte").into()
            }
            (BinaryOp::Add, ExpTypeVal::Int(_), ExpTypeVal::Int(_)) => {
                self.bd.build_int_add(lval, rval, "add").into()
            }
            (BinaryOp::Sub, ExpTypeVal::Int(_), ExpTypeVal::Int(_)) => {
                self.bd.build_int_sub(lval, rval, "sub").into()
            }
            (BinaryOp::Mul, ExpTypeVal::Int(_), ExpTypeVal::Int(_)) => {
                self.bd.build_int_mul(lval, rval, "mul").into()
            }
            (BinaryOp::Mod, ExpTypeVal::Int(_), ExpTypeVal::Int(_)) => {
                self.bd.build_int_signed_rem(lval, rval, "mod").into()
            }
            (BinaryOp::Div, ExpTypeVal::Int(_), ExpTypeVal::Int(_)) => {
                self.bd.build_int_signed_div(lval, rval, "div").into()
            }
            _ => panic!("unexpected binary expression")
        }
    }

    fn compile_exp(&self, node: &ExpNode) -> Option<BasicValueEnum<'llvm>> {
        // TODO: if node has typeval with value, return literal :)
        match &node.exp {
            Exp::Call(ident, args) => {
                let fnval = *self.fenv.get(ident).unwrap();
                let argsvals: Vec<BasicValueEnum> = args.iter().map(|x| self.compile_exp(x).unwrap()).collect();
                let result = self.bd.build_call(fnval, &argsvals, "").try_as_basic_value();
                match result {
                    Either::Left(l) => Some(l),
                    Either::Right(_) => None,
                }
            },
            Exp::Var(ident) => Some(*self.venv.get(ident).unwrap()),
            Exp::Int(val) => Some(self.llvm.i32_type().const_int(*val as u64, false).into()),
            Exp::Bool(val) => Some(self.llvm.bool_type().const_int(*val as u64, false).into()),
            Exp::Unary(op, exp) => {
                let val = self.compile_exp(exp).unwrap();
                match op {
                    UnaryOp::Neg => Some(self.bd.build_int_neg(val.into_int_value(), "neg").into()),
                    UnaryOp::Not => Some(self.bd.build_not(val.into_int_value(), "not").into()),
                }
            }
            Exp::Binary(lexp, op, rexp) => Some(self.compile_bin_exp(op, lexp, rexp)),
            Exp::Str(_) => unimplemented!()
        }
    }

    fn compile_stmt(&mut self, node: &StmtNode) {
        match &node.stmt {
            Stmt::BStmt(stmts) => {
                for stmt in stmts {
                    self.compile_stmt(stmt);
                }
            }
            Stmt::Decl(decl) => {
//                let ttype = self.get_llvm_basic_type(&decl.type_spec.ttype).unwrap();
                for body in &decl.vars {
                    let init_val = match &body.init {
                        Some(exp) => self.compile_exp(exp).unwrap(),
                        None => self.get_llvm_default_value(&decl.type_spec.ttype).unwrap()
                    };
                    self.venv.insert(body.ident.clone(), init_val);
                }
            }
            Stmt::Ass(ident, exp) => {
                let val = self.compile_exp(exp).unwrap();
                self.venv.insert(ident.clone(), val).unwrap();

            }
            Stmt::Incr(ident) => {
                let var = self.venv.get(ident).unwrap().into_int_value();
                let one = self.llvm.i32_type().const_int(1, false);
                let val = self.bd.build_int_add(var, one, "").into();
                self.venv.insert(ident.clone(), val);
            }
            Stmt::Decr(ident) => {
                let var = self.venv.get(ident).unwrap().into_int_value();
                let one = self.llvm.i32_type().const_int(1, false);
                let val = self.bd.build_int_sub(var, one, "").into();
                self.venv.insert(ident.clone(), val);
            }
            Stmt::EStmt(exp_node) => {
                self.compile_exp(exp_node);
            },
            Stmt::Ret(node) => {
                self.bd.build_return(Some(&self.compile_exp(node).unwrap()));
            }
            Stmt::VRet => {
                self.bd.build_return(None);
            }
        }
    }

    fn compile_fndef(&mut self, fndef: &FnDef) {
        let fnval = *self.fenv.get(&fndef.ident).unwrap();
        let entry = self.llvm.append_basic_block(fnval, "entry");
        self.venv = VEnv::new();
        for (i, param) in fndef.params.iter().enumerate() {
            let name = &param.vars.first().unwrap().ident;
            let val = fnval.get_nth_param(i as u32).unwrap();
            self.venv.insert(name.clone(), val);
        }
        self.bd.position_at_end(&entry);
        self.compile_stmt(&fndef.body);
    }

    fn compile_fndecl(&mut self, ident: &Ident, signature: &FnSignature){
        let (ret_type, args_types) = signature;
        let arg_types: Vec<BasicTypeEnum> = args_types.iter().map(|x| self.get_llvm_basic_type(x).unwrap()).collect();
        let fn_type = match ret_type {
            Type::Void => self.llvm.void_type().fn_type(&arg_types, false),
            _ => self.get_llvm_basic_type(&ret_type).unwrap().fn_type(&arg_types, false),
        };
        let fnval = self.md.add_function(ident, fn_type, None);
        self.fenv.insert(ident.clone(), fnval);
    }

    fn compile_prog(&mut self, prog: &Program) {
        self.compile_fndecl(&"readInt".to_owned(), &(Type::Int, vec![]));
        self.compile_fndecl(&"readString".to_owned(), &(Type::Str, vec![]));
        self.compile_fndecl(&"printInt".to_owned(), &(Type::Void, vec![Type::Int]));
        self.compile_fndecl(&"printString".to_owned(), &(Type::Void, vec![Type::Str]));

        for fndef in &prog.functions {
            self.compile_fndecl(&fndef.ident, &fndef.get_signature());
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
    let rt_buffer = MemoryBuffer::create_from_file(Path::new("lib/runtime.ll")).unwrap();
    let rt_mod = backend.llvm.create_module_from_ir(rt_buffer).unwrap();
    backend.md.link_in_module(rt_mod).unwrap();
    backend.md.print_to_file("simplest.ll").unwrap();
    backend.md.write_bitcode_to_file(&fs::File::create("simplest.bc").unwrap(), true, false);
    println!("{}", backend.md.print_to_string().to_string());
}
