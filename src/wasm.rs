use parity_wasm::elements::{Module, Instruction, Internal, ValueType};
use std::convert::TryFrom;

pub fn load(wasm_file: &str) -> Result<Module, Box<dyn std::error::Error>> {
    let module = parity_wasm::deserialize_file(wasm_file)?;
    println!("module: {:?}", module);
    Ok(module)
}

pub fn func_from_name<'a>(module: &'a Module, func: &'a str) -> Result<usize, Box<dyn std::error::Error>> {
    let export_section = module.export_section().ok_or("missing export section")?;
    let entry = export_section
        .entries()
        .iter()
        .find(|entry| entry.field() == func ).ok_or(format!("no exported func {}", func))?;
    let func_num = match entry.internal() {
        &Internal::Function(v) => usize::try_from(v)?,
        _ => panic!("exported function was not function")
    };

    Ok(func_num)
}

pub fn parse<'a>(module: &'a Module, func_num: usize) -> Result<(&'a [ValueType], &'a [Instruction]), Box<dyn std::error::Error>> {
    let type_section = module.type_section().ok_or("wasm missing type section")?;
    let function_section = module.function_section().ok_or("wasm missing function section")?;
    let func_type = function_section.entries().get(func_num).ok_or("missing func type def")?.type_ref();
    let func_type_us = usize::try_from(func_type)?;
    let t = &type_section.types()[func_type_us];
    let func_t = match t {
        parity_wasm::elements::Type::Function(x) => x
    };
    let code_section = module.code_section().ok_or("wasm missing code section")?;
    let func = &code_section.bodies()[func_num];
    let params = func_t.params();
    let elements = func.code().elements();

    Ok((params, elements))
}
