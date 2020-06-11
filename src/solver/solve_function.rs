use crate::wasm;
use parity_wasm::elements::Instruction;
use parity_wasm::elements::{Instruction::*, Module, ValueType};
use z3::{Solver, Context, ast::{Ast, Int}};use std::convert::TryFrom;
use log::{warn};
use std::collections::HashMap;

fn op2<'a, 'b>(stack: &'b mut Vec<Int<'a>>, solver: &'a Solver, ctx: &'a Context, i: usize, op: &dyn Fn(&Int<'b>, &Int<'b>) -> Int<'b>) -> Result<(), Box<dyn std::error::Error>> {
    let x = stack.pop().ok_or("missing op 1")?;
    let y = stack.pop().ok_or("missing op 2")?;
    let r = Int::new_const(&ctx, format!("op{}", i));
    solver.assert(&r._eq(&op(&x, &y)));
    stack.push(r);
    Ok(())
}

fn op3<'a, 'b>(stack: &'b mut Vec<Int<'a>>, solver: &'a Solver, ctx: &'a Context, i: usize, op: &dyn Fn(&Int<'b>, &Int<'b>, &Int<'b>) -> Int<'b>) -> Result<(), Box<dyn std::error::Error>> {
    let x = stack.pop().ok_or("missing op 1")?;
    let y = stack.pop().ok_or("missing op 2")?;
    let z = stack.pop().ok_or("missing op 3")?;
    let r = Int::new_const(&ctx, format!("op{}", i));
    solver.assert(&r._eq(&op(&x, &y, &z)));
    stack.push(r);
    Ok(())
}

pub fn solve_function<'a, 'b>(stack: &'b mut Vec<Int<'a>>, solver: &'a Solver, ctx: &'a Context, module: &'a Module, code: &[Instruction], globals: &[Int<'a>], locals: &[Int<'a>], mem: &'b HashMap<usize, Int<'a>>) -> Result<(), Box<dyn std::error::Error>> {
    let one = Int::from_i64(&ctx, 1);
    let zero = Int::from_i64(&ctx, 0);

    warn!("Solving Function\nLocals: {:?}\nInstructions: {:?}", locals, code);

    // Now let's do some simple solving
    for (i, op) in code.iter().enumerate() {
        match op {
            &GetGlobal(v) => {
                let v_us = usize::try_from(v)?;
                println!("global {} {:?}", v_us, globals[v_us]);
                stack.push(globals[v_us].clone())
            }
            &GetLocal(v) => {
                let v_us = usize::try_from(v)?;
                println!("local {} {:?}", v_us, locals[v_us]);
                stack.push(locals[v_us].clone())
            }
            &TeeLocal(v) => {
                let v_us = usize::try_from(v)?;
                println!("local {} {:?}", v_us, locals[v_us]);
                let val = stack.pop().ok_or("missing op")?;
                solver.assert(&val._eq(&locals[v_us]));
                stack.push(locals[v_us].clone())
            }
            &SetLocal(v) => {
                let v_us = usize::try_from(v)?;
                println!("local {} {:?}", v_us, locals[v_us]);
                let val = stack.pop().ok_or("missing op")?;
                solver.assert(&val._eq(&locals[v_us]));
            }
            &I32Load(_, v) => {
                let v_us = usize::try_from(v)?;
                let val = mem.get(&v_us).map(|x|x.clone()).unwrap_or(Int::from_i64(&ctx, 0));
                println!("memload {} {:?}", v_us, val);
                stack.push(val)
            }
            &I32Const(x) => {
                // Push a const value onto the stack
                stack.push(Int::from_i64(&ctx, x as i64))
            }
            I32Add => {
                op2(stack, &solver, &ctx, i, &|x: &Int, y: &Int| y.add(&[&x]))?
            }
            I32Sub => {
                op2(stack, &solver, &ctx, i, &|x: &Int, y: &Int| y.sub(&[&x]))?
            }
            I32Mul => {
                op2(stack, &solver, &ctx, i, &|x: &Int, y: &Int| y.mul(&[&x]))?
            }
            I32GtS => {
                op2(stack, &solver, &ctx, i, &|x: &Int, y: &Int| y.gt(&x).ite(&one, &zero))?
            }
            I32GeS => {
                op2(stack, &solver, &ctx, i, &|x: &Int, y: &Int| y.ge(&x).ite(&one, &zero))?
            }
            Select => {
                op3(stack, &solver, &ctx, i, &|x: &Int, y: &Int, z: &Int| x._eq(&zero).not().ite(&z, &y))?
            }
            &Call(v) => {
                let new_func_num = usize::try_from(v).unwrap();
                let (func_params, func_code) = wasm::parse(module, new_func_num)?;
                let inputs = func_params.iter().enumerate().map(|(i, param)|
                    match param {
                        ValueType::I32 => {
                            let i = Int::new_const(&ctx, format!("local_f{}_{}", new_func_num, i));
                            let input = stack.pop().expect("must have value");
                            solver.assert(&i._eq(&input));
                            Ok(i)
                        },
                        _ => Err(format!("not handled local type, {:?}", param))?
                    }
                ).collect::<Result<Vec<Int>, Box<dyn std::error::Error>>>()?;
                solve_function(stack, solver, ctx, module, func_code, globals, &inputs, &mem)?;
            }
            End => {
                // Nothing
            }
            _ =>
                return Err(format!("not handled wasm code {:?}", op).into())
        }
    }

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         test_solve(5, vec![GetLocal(0)])
//         assert_eq!(2 + 2, 4);
//     }
// }