mod solve_function;
use std::collections::HashMap;
use crate::wasm;
use parity_wasm::elements::{Module, ValueType};
use z3::{Solver, Config, Context, SatResult, ast::{Ast, BV, Int}};

pub fn solve(module: &Module, func_num: usize) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    let mut stack: Vec<Int> = vec![];

    let (params, code) = wasm::parse(module, func_num)?;
    let globals = module.global_section().map(|s|s.entries()).unwrap_or(&[]).iter().enumerate().map(|(i,global)|
        // TODO: Handle mutable and init expressions
        match global.global_type().content_type() {
            ValueType::I32 => Ok(Int::new_const(&ctx, format!("global_{}", i))),
            t => Err(format!("not handled global type, {:?}", t))?
        }
    ).collect::<Result<Vec<Int>, Box<dyn std::error::Error>>>()?;

    let inputs = params.iter().enumerate().map(|(i, param)|
        match param {
            ValueType::I32 => Ok(Int::new_const(&ctx, format!("input_{}", i))),
            _ => Err(format!("not handled local type, {:?}", param))?
        }
    ).collect::<Result<Vec<Int>, Box<dyn std::error::Error>>>()?;
    // TODO: Handle BV size changes (how?)
    let mem: BV = ;

    solve_function::solve_function(&mut stack, &solver, &ctx, module, code, &globals, &inputs, &mem)?;
    let final_stack = stack.pop().ok_or("expected single stack result")?;
    let result = Int::new_const(&ctx, "result");
    solver.assert(&result._eq(&final_stack));
    let zero = Int::from_i64(&ctx, 0);
    solver.assert(&inputs[0]._eq(&zero).not());

    // Now, let's build the expectation, currently by hand
    let expected = Int::new_const(&ctx, "expected");
    solver.assert(&expected._eq(&Int::from_i64(&ctx, 10)));
    solver.assert(&result._eq(&expected).not());
    println!("Solver:\n{}", solver);
    
    assert_eq!(solver.check(), SatResult::Sat);
    println!("Solution:\n{}", solver.get_model());

    Ok(())
}
