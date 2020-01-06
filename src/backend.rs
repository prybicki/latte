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
use inkwell::basic_block::BasicBlock;

type FEnv<'llvm> = HashMap<Ident, FunctionValue<'llvm>>;
type VEnv<'llvm> = ScopedMap<Ident, BasicValueEnum<'llvm>>; // value env
type TEnv<'llvm> = ScopedMap<Ident, Type>; // type env

// TODO: Backend is quietly assuming that condition expressions have no side effects (other than printing)

struct Backend<'llvm> {
    llvm: &'llvm Context,
    md: Module<'llvm>,
    bd: Builder<'llvm>,
    fenv: FEnv<'llvm>,
    venv: VEnv<'llvm>,
    tenv: TEnv<'llvm>,
    curr_fn: Option<FunctionValue<'llvm>>,
}

trait HasSetName {
    fn set_name(&self, s: &str);
}

impl<'llvm> HasSetName for BasicValueEnum<'llvm> {
    fn set_name(&self, s: &str) {
        match self {
            BasicValueEnum::ArrayValue(v) => v.set_name(s),
            BasicValueEnum::IntValue(v) => v.set_name(s),
            BasicValueEnum::FloatValue(v) => v.set_name(s),
            BasicValueEnum::PointerValue(v) => v.set_name(s),
            BasicValueEnum::StructValue(v) => v.set_name(s),
            BasicValueEnum::VectorValue(v) => v.set_name(s),
        }
    }
}

impl<'llvm> Backend<'llvm> {
    fn new(llvm: &'llvm Context, mod_name: &str) -> Backend<'llvm> {
        let md = llvm.create_module(mod_name);
        let bd = llvm.create_builder();
        let fenv = FEnv::new();
        let venv = VEnv::new();
        let tenv = TEnv::new();
        Backend {llvm, md, bd, fenv, venv, tenv, curr_fn: None}
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
                self.venv.push_scope();
                for stmt in stmts {
                    self.compile_stmt(stmt);
                }
                self.venv.pop_scope();
            }
            Stmt::Decl(decl) => {
                for body in &decl.vars {
                    let init_val = match &body.init {
                        Some(exp) => self.compile_exp(exp).unwrap(),
                        None => self.get_llvm_default_value(&decl.type_spec.ttype).unwrap()
                    };
                    init_val.set_name(&body.ident);
                    self.tenv.insert_into_top_scope(body.ident.clone(), decl.type_spec.ttype);
                    self.venv.insert_into_top_scope(body.ident.clone(), init_val);
                }
            }
            Stmt::Ass(ident, exp) => {
                let val = self.compile_exp(exp).unwrap();
                val.set_name(ident);
                self.venv.replace_topmost(ident.clone(), val);
            }
            Stmt::Incr(ident) => {
                let var = self.venv.get(ident).unwrap().into_int_value();
                let one = self.llvm.i32_type().const_int(1, false);
                let val = self.bd.build_int_add(var, one, "").into();
                self.venv.replace_topmost(ident.clone(), val);
            }
            Stmt::Decr(ident) => {
                let var = self.venv.get(ident).unwrap().into_int_value();
                let one = self.llvm.i32_type().const_int(1, false);
                let val = self.bd.build_int_sub(var, one, "").into();
                self.venv.replace_topmost(ident.clone(), val);
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
            Stmt::Cond(cond, tstmt, fstmt) => {
                let fnval = self.curr_fn.unwrap();

                // TODO make it lazy

                let tstmt_returns = tstmt.ret.unwrap();
                let fstmt_returns = if let Some(fstmt) = fstmt { fstmt.ret.unwrap() } else { false };
                let non_returning_blocks = !tstmt_returns as i32 + !fstmt_returns as i32;

                // create basic block for all statements, they may end up being empty
                let cond = self.compile_exp(cond).unwrap().into_int_value();

                let pred_block = self.bd.get_insert_block().unwrap();
                let then_block = self.llvm.append_basic_block(fnval, "then");
                let cont_block = self.llvm.append_basic_block(fnval, "cont");
                let else_block = self.llvm.append_basic_block(fnval, "else");

                // remember values before branching
                let pred_venv = self.venv.clone();

                // branch
                self.bd.build_conditional_branch(
                    cond,
                    &then_block,
                    if fstmt.is_none() { &cont_block } else { &else_block }
                );

                // build true-statement block
                self.bd.position_at_end(&then_block);
                self.compile_stmt(tstmt);
                // if true-statement does not return, jump to continuation block
                if !tstmt.ret.unwrap() {
                    self.bd.build_unconditional_branch(&cont_block);
                }
                // expect to know the same set of variables after compiling true statment, possibly with different values
                assert!(self.venv.keys().eq(pred_venv.keys()));

                // remember variables after true-statement and after optional false-statement
                let then_venv = self.venv.clone();
                let mut else_venv = None; // just a placeholder here

                // build false-statement block
                if let Some(fstmt) = fstmt {
                    // roll back variables to cancell effects of compilation of true statement
                    self.venv.clone_from(&pred_venv);
                    self.bd.position_at_end(&else_block);
                    self.compile_stmt(fstmt);
                    // if false-statment does not return, jump to continuation block
                    if !fstmt.ret.unwrap() {
                        self.bd.build_unconditional_branch(&cont_block);
                    }
                    // expect to know the same set of variables after compiling true statment, possibly with different values
                    assert!(self.venv.keys().eq(pred_venv.keys()));
                    // remember variables
                    else_venv = Some(self.venv.clone());
                }

                // cont block is necessary
                if non_returning_blocks > 0 {
                    self.bd.position_at_end(&cont_block);

                    for var in pred_venv.keys() {
                        let mut entries: Vec<(&dyn BasicValue, &BasicBlock)> = Vec::new();
                        // TODO simplify it?

                        // one of the block does not return, so must be one of the predecessors
                        if fstmt.is_none() {
                            let pred_val = pred_venv.get(var).unwrap();
                            entries.push((pred_val, &pred_block));
                        }

                        // true-statement precedes if it does not return
                        if !tstmt_returns { // tstmt does not return
                            let then_val = then_venv.get(var).unwrap();
                            entries.push((then_val, &then_block));
                        }
                        // false-statment precedes if it exists and does not return
                        if fstmt.is_some() && !fstmt_returns { // fstmt exists and does not return
                            let else_val = else_venv.as_ref().unwrap().get(var).unwrap();
                            entries.push((else_val, &else_block));
                        }

                        assert_ne!(entries.len(), 0);
                        if entries.len() == 1 {
                            let val = entries.first().unwrap().0;
                            self.venv.replace_topmost(var.clone(), val.as_basic_value_enum());
                        }
                        else {
                            let ttype = self.get_llvm_basic_type(self.tenv.get(var).unwrap()).unwrap();
                            let phi = self.bd.build_phi(ttype, var);
                            phi.add_incoming(entries.as_slice());
                            self.venv.replace_topmost(var.clone(), phi.as_basic_value());
                        }
                    }
                }
            }
            Stmt::While(cond, body) => {
                let fnval = self.curr_fn.unwrap();
                let pred_block = self.bd.get_insert_block().unwrap();
                let pred_venv = self.venv.clone();

                // TODO make it lazy
                let body_returns = body.ret.unwrap();
                let cond_block = self.llvm.append_basic_block(fnval, "loop_cond");
                let body_block = self.llvm.append_basic_block(fnval, "loop_body");
                let cont_block = self.llvm.append_basic_block(fnval, "loop_cont");
                self.bd.build_unconditional_branch(&cond_block);

                let mut phi_venv = HashMap::<Ident, PhiValue>::new();

                // build condition (preds = pred, body)
                {
                    self.bd.position_at_end(&cond_block);

                    if !body_returns {
                        // build phi placeholders
                        for var in self.tenv.keys() {
                            let ttype = self.get_llvm_basic_type(self.tenv.get(var).unwrap()).unwrap();
                            let phi = self.bd.build_phi(ttype, var);

                            // does it work?
                            phi_venv.insert(var.clone(), phi.clone());
                            println!("replacing {}", var);
                            self.venv.replace_topmost(var.clone(), phi.as_basic_value());
                        }
                    }
                    let cond_val = self.compile_exp(cond).unwrap().into_int_value();
                    self.bd.build_conditional_branch(cond_val, &body_block, &cont_block);
                }
                let cond_venv = self.venv.clone();

                // build body
                {
                    self.bd.position_at_end(&body_block);
                    self.compile_stmt(body);
                    if !body_returns {
                        self.bd.build_unconditional_branch(&cond_block);
                    }
                }

                if !body_returns {
                    // build actual phi values in cond block
                    for var in self.tenv.keys() {
                        let phi = phi_venv.get(var).unwrap();
                        let pred_val = pred_venv.get(var).unwrap();
                        let body_val = self.venv.get(var).unwrap();
                        phi.add_incoming(&[(pred_val, &pred_block), (body_val, &body_block)]);
                        self.venv.replace_topmost(var.clone(), phi.as_basic_value());
                    }
                    self.bd.position_at_end(&cont_block);
                }
            }
        }
    }

    fn compile_fndef(&mut self, fndef: &FnDef) {
        let fnval = *self.fenv.get(&fndef.ident).unwrap();
        self.curr_fn = Some(fnval);
        let entry = self.llvm.append_basic_block(fnval, "entry");
        self.venv = VEnv::new();
        self.tenv = TEnv::new();
        for (i, param) in fndef.params.iter().enumerate() {
            let name = &param.vars.first().unwrap().ident;
            let val = fnval.get_nth_param(i as u32).unwrap();
            self.venv.insert_into_top_scope(name.clone(), val);
            self.tenv.insert_into_top_scope(name.clone(), param.type_spec.ttype);
        }
        self.bd.position_at_end(&entry);
        self.compile_stmt(&fndef.body);
        self.remove_empty_basic_blocks();
    }

    fn remove_empty_basic_blocks(&mut self) {
        let mut ff = self.md.get_first_function();
        while let Some(function) = ff {

            let mut bb = function.get_first_basic_block();
            while let Some(basic_block) = bb {
                if let None = basic_block.get_first_instruction() {
                    basic_block.remove_from_function();
                    println!("removed {:?} / {:?}", function, basic_block);
                }

                bb = basic_block.get_next_basic_block();
            }

            ff = function.get_next_function();
        }
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
    if let Err(e) = backend.md.verify() {
        println!("Errors:\n{}", e.to_string());
    }
    backend.md.write_bitcode_to_file(&fs::File::create("simplest.bc").unwrap(), true, false);
}
