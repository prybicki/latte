use crate::scoped_map::ScopedMap;
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
use inkwell::basic_block::BasicBlock;
use inkwell::support::LLVMString;

type FEnv<'llvm> = HashMap<Ident, FunctionValue<'llvm>>;
type VEnv<'llvm> = ScopedMap<Ident, BasicValueEnum<'llvm>>; // value env
type TEnv<'llvm> = ScopedMap<Ident, Type>; // type env
type SEnv<'llvm> = HashMap<String, GlobalValue<'llvm>>;

// TODO: Backend is quietly assuming that condition expressions have no side effects (other than printing)

struct Backend<'llvm> {
    llvm: &'llvm Context,
    md: Module<'llvm>,
    bd: Builder<'llvm>,
    fenv: FEnv<'llvm>,
    venv: VEnv<'llvm>,
    tenv: TEnv<'llvm>,
    senv: SEnv<'llvm>,

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
        let senv = SEnv::new();
        Backend {llvm, md, bd, fenv, venv, tenv, senv, curr_fn: None}
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
            Type::Str => Some(self.llvm.i8_type().const_array(&[self.get_llvm_default_value(&Type::Int).unwrap().into_int_value()]).into()),
            Type::Void => None
        }
    }

    fn compile_bin_exp(&mut self, op: &BinaryOp, lexp: &ExpNode, rexp: &ExpNode) -> BasicValueEnum<'llvm> {
        let ltv = lexp.typeval.as_ref().unwrap();
        let rtv = rexp.typeval.as_ref().unwrap();
        // string evaluation
        if let (BinaryOp::Add, ExpTypeVal::Str(_), ExpTypeVal::Str(_)) = (op, ltv, rtv) {
            let lval = self.compile_exp(lexp).unwrap().into_pointer_value();
            let rval = self.compile_exp(rexp).unwrap().into_pointer_value();

            let fnval = *self.fenv.get("__latc_concat_str").unwrap();
            let argsvals: Vec<BasicValueEnum> = vec![lval.into(), rval.into()];
            let result = self.bd.build_call(fnval, &argsvals, "").try_as_basic_value();
            result.left().expect("got void from __latc_concat_str?")
        }
        else {
            // lazy evaluation
            match (op, ltv, rtv) {
                (BinaryOp::Or, ExpTypeVal::Bool(_), ExpTypeVal::Bool(_)) => {
                    let curr_fn = self.curr_fn.unwrap();

                    let current_bb = self.bd.get_insert_block().unwrap();
                    let le_lhs_false = self.llvm.append_basic_block(curr_fn, "lazy_eval_or_lhs_false");
                    let le_done = self.llvm.append_basic_block(curr_fn, "lazy_eval_or_done");

                    // evaluate lhs and branch to "done" or rhs evaluation depending whether lhs was conclusive or not
                    let lhs_val = self.compile_exp(lexp).unwrap().into_int_value();
                    self.bd.build_conditional_branch(
                        lhs_val,
                        &le_done,
                        &le_lhs_false
                    );

                    // for the case lhs was non-conclusive, emit rhs evaluation in separate block
                    self.bd.position_at_end(&le_lhs_false);
                    let rhs_val = self.compile_exp(rexp).unwrap().into_int_value();
                    // rhs may be also evaluated lazily, so remember it's "done" block
                    let rhs_end_block = self.bd.get_insert_block().unwrap();
                    self.bd.build_unconditional_branch(&le_done);

                    // build done
                    self.bd.position_at_end(&le_done);
                    let phi = self.bd.build_phi(self.llvm.bool_type(), "lazy_eval_or_result");
                    let true_value = self.llvm.bool_type().const_int(1, false);
                    phi.add_incoming(&[(&true_value, &current_bb), (&rhs_val, &rhs_end_block)]);

                    phi.as_basic_value()
                }
                (BinaryOp::And, ExpTypeVal::Bool(_), ExpTypeVal::Bool(_)) => {
                    // SEE OR FOR COMMENTS
                    let curr_fn = self.curr_fn.unwrap();
                    let current_bb = self.bd.get_insert_block().unwrap();
                    let le_lhs_true = self.llvm.append_basic_block(curr_fn, "lazy_eval_and_lhs_true");
                    let le_done = self.llvm.append_basic_block(curr_fn, "lazy_eval_and_done");

                    let lhs_val = self.compile_exp(lexp).unwrap().into_int_value();
                    self.bd.build_conditional_branch(
                        lhs_val,
                        &le_lhs_true,
                        &le_done
                    );

                    self.bd.position_at_end(&le_lhs_true);
                    let rhs_val = self.compile_exp(rexp).unwrap().into_int_value();
                    let rhs_end_block = self.bd.get_insert_block().unwrap();
                    self.bd.build_unconditional_branch(&le_done);

                    self.bd.position_at_end(&le_done);
                    let phi = self.bd.build_phi(self.llvm.bool_type(), "lazy_eval_and_result");
                    let false_value = self.llvm.bool_type().const_int(0, false);
                    phi.add_incoming(&[(&false_value, &current_bb), (&rhs_val, &rhs_end_block)]);

                    phi.as_basic_value()
                }
                // eager evaluation
                _ => {
                    let lval = self.compile_exp(lexp).unwrap().into_int_value();
                    let rval = self.compile_exp(rexp).unwrap().into_int_value();
                    match (op, ltv, rtv) {
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
            }
        }
    }

    fn compile_exp(&mut self, node: &ExpNode) -> Option<BasicValueEnum<'llvm>> {
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
            Exp::Str(str_exp) => {

                let global_ptr = match self.senv.get(str_exp) {
                    None => {
                        let str_val = self.llvm.const_string(str_exp.as_bytes(), true);
                        let global = self.md.add_global(str_val.get_type(), None, "str_lit");
                        global.set_initializer(&str_val);
                        self.senv.insert(str_exp.clone(), global);
                        self.senv.get(str_exp).unwrap()
                    },
                    Some(g) => g,
                }.as_pointer_value();

                let zero = self.get_llvm_default_value(&Type::Int).unwrap().into_int_value();
                unsafe {
                    Some(self.bd.build_gep(global_ptr, &[zero, zero], "").into())
                }
            }
        }
    }

    fn compile_nontrivial_cond_stmt(&mut self, cond: &Box<ExpNode>, tstmt: &Box<StmtNode>, fstmt: &Option<Box<StmtNode>>, node_will_return: bool) {
        let curr_fn = self.curr_fn.unwrap();

        // create basic block for all statements, they may end up being empty
        let pred_block = self.bd.get_insert_block().unwrap();
        let then_block = self.llvm.append_basic_block(curr_fn, "then");
        let cont_block = self.llvm.append_basic_block(curr_fn, "cont");
        let else_block = self.llvm.append_basic_block(curr_fn, "else");

        let then_returns_if_entered = node_will_return || tstmt.will_return.unwrap();
        let else_returns_if_entered = match fstmt {
            None => false,
            Some(fstmt) => node_will_return || fstmt.will_return.unwrap(),
        };
        let pred_venv = self.venv.clone();

        let cond_val = self.compile_exp(cond).unwrap().into_int_value();

        self.bd.build_conditional_branch(
            cond_val,
            &then_block,
            if fstmt.is_none() { &cont_block } else { &else_block }
        );

        // build true-statement block
        self.bd.position_at_end(&then_block);
        self.compile_stmt(tstmt);
        if !then_returns_if_entered {
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

            if !else_returns_if_entered {
                self.bd.build_unconditional_branch(&cont_block);
            }
            // expect to know the same set of variables after compiling true statment, possibly with different values
            assert!(self.venv.keys().eq(pred_venv.keys()));
            // remember variables
            else_venv = Some(self.venv.clone());
        }

        // cont block is necessary
        if !node_will_return {
            self.bd.position_at_end(&cont_block);

            for var in pred_venv.keys() {
                let mut entries: Vec<(&dyn BasicValue, &BasicBlock)> = Vec::new();

                // no else means we take values from pred
                if fstmt.is_none() {
                    let pred_val = pred_venv.get(var).unwrap();
                    entries.push((pred_val, &pred_block));
                }

                if !then_returns_if_entered {
                    let then_val = then_venv.get(var).unwrap();
                    entries.push((then_val, &then_block));

                }

                if fstmt.is_some() && !else_returns_if_entered {
                    let else_val = else_venv.as_ref().unwrap().get(var).unwrap();
                    entries.push((else_val, &else_block));
                }

                let ttype = self.get_llvm_basic_type(self.tenv.get(var).unwrap()).unwrap();
                let phi = self.bd.build_phi(ttype, var);
                phi.add_incoming(entries.as_slice());
                self.venv.replace_topmost(var.clone(), phi.as_basic_value());
            }
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
                let exp = self.compile_exp(node).unwrap();
                self.bd.build_return(Some(&exp));
            }
            Stmt::VRet => {
                self.bd.build_return(None);
            }
            Stmt::Cond(cond, tstmt, fstmt) => {
                match cond.typeval.as_ref().unwrap() {
                    ExpTypeVal::Bool(Some(true)) => self.compile_stmt(tstmt),
                    ExpTypeVal::Bool(Some(false)) => {
                        if let Some(stmt) = fstmt {
                            self.compile_stmt(stmt)
                        }
                    }
                    ExpTypeVal::Bool(None) => {
                        self.compile_nontrivial_cond_stmt(cond, tstmt, fstmt, node.will_return.unwrap())
                    }
                    _ => panic!("backend: invalid type in condition")
                }

            }
            Stmt::While(cond, body) => {
                let fnval = self.curr_fn.unwrap();
                let pred_block = self.bd.get_insert_block().unwrap();
                let pred_venv = self.venv.clone();

                let body_returns = body.will_return.unwrap();
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
                            self.venv.replace_topmost(var.clone(), phi.as_basic_value());
                        }
                    }
                    let cond_val = self.compile_exp(cond).unwrap().into_int_value();
                    self.bd.build_conditional_branch(cond_val, &body_block, &cont_block);
                }

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
                let next = basic_block.get_next_basic_block();
                if let None = basic_block.get_first_instruction() {
                    basic_block.remove_from_function().expect("error while removing unused basic blocks");
                }
                bb = next;

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
        self.compile_fndecl(&"__latc_concat_str".to_owned(), &(Type::Str, vec![Type::Str, Type::Str]));

        for fndef in &prog.functions {
            self.compile_fndecl(&fndef.ident, &fndef.get_signature());
        }

        for fndef in &prog.functions {
            self.compile_fndef(fndef);

        }
    }
}

pub fn compile(prog: &Program, path: &Path) -> Result<(), LLVMString> {
    // split path
    let mod_name = path.file_name().unwrap().to_str().unwrap().to_owned();
    let dir_path = path.parent().unwrap_or(Path::new("."));

    // init things
    let llvm = Context::create();
    let mut backend = Backend::new(&llvm, &mod_name);

    // load runtime
    let rt_buffer = MemoryBuffer::create_from_file(Path::new("lib/runtime.ll")).unwrap();
    let rt_mod = backend.llvm.create_module_from_ir(rt_buffer).unwrap();

    // compile & link
    backend.compile_prog(prog);
    backend.md.link_in_module(rt_mod).unwrap();

    // handle result
    match backend.md.verify() {
        Ok(_) => {
            backend.md.print_to_file(dir_path.join(Path::new(&(mod_name.clone() + ".ll"))))?;
            backend.md.write_bitcode_to_path(dir_path.join(Path::new(&(mod_name.clone() + ".bc"))).as_path());
            Ok(())
        },
        Err(e) => Err(e)
    }
}
