// https://bouvier.cc/2021/02/17/cranelift-codegen-primer/

type ResultError = Box<dyn std::error::Error + Send + Sync>;
type ResultReturn = Result<(), ResultError>;


fn attempt_two() -> ResultReturn {
    // use crate::frontend::*;
    // use cranelift::prelude::*;
    // use cranelift_jit::{JITBuilder, JITModule};
    // For jit builder:
    // let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    // let mut module = JITModule::new(builder);
    /*
    // module. finalize_definitions()?;
    let ptr = module.get_finalized_function(id);
    unsafe {
        let z = std::slice::from_raw_parts(ptr, 5);
        println!("z: {z:x?}");
    }
    */
    // use cranelift_codegen::settings::Configurable;
    use cranelift_codegen::settings::Flags;
    use cranelift_module::{Linkage, Module};

    // https://github.com/bytecodealliance/cranelift-jit-demo/blob/main/src/jit.rs

    let flag_builder = cranelift_codegen::settings::builder();

    let autoselect = false;
    // let autoselect = true;

    // let isa_name_to_use = "aarch64";
    let isa_name_to_use = "x86_64"; // Panics on debug assert cranelift-codegen-0.116.1/src/isa/x64/lower.rs:233

    let isa = if autoselect {
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        // let isa = isa_builder
        isa_builder.finish(Flags::new(flag_builder)).unwrap()
    } else {
        let flags = Flags::new(flag_builder);
        println!("flags: {}", &flags);

        cranelift_codegen::isa::lookup_by_name(isa_name_to_use)?.finish(flags)?
    };

    println!("isa.triple(): {}", &isa.triple());
    println!("isa: {:#}", &isa);

    let builder = cranelift_object::ObjectBuilder::new(
        isa,
        "hello",
        cranelift_module::default_libcall_names(),
    )?;
    let mut module = cranelift_object::ObjectModule::new(builder);

    // let f = std::fs::read_to_string("./test/average.clif").map_err(|e| format!("could not find file ./test/average.clif: {e:?}"))?;
    // let f = std::fs::read_to_string("./test/average_fixed.clif").map_err(|e| format!("could not find file ./test/average.clif: {e:?}"))?;
    let f = std::fs::read_to_string("./test/average_small.clif").map_err(|e| format!("could not find file ./test/average.clif: {e:?}"))?;

    let mut fun = cranelift_reader::parse_functions(&f)?;
    let fun = fun.drain(..).next().unwrap();


    let stencil = &fun.stencil;
    let dfg = &stencil.dfg;
    let layout = &stencil.layout;

    // let mut buffer = vec![];

    use cranelift_codegen::ir;
    for b in layout.blocks() {
        println!("b: {b:?}");
        let block_data = &dfg.blocks[b];
        println!(
            "block_data params: {:?}",
            block_data.params(&dfg.value_lists)
        );
        for inst in layout.block_insts(b) {
            println!("inst: {inst:?}");
            let instdata = dfg.insts[inst];
            println!("  instruction_data: {instdata:?}");
            println!("  typevar_operand: {:?}", instdata.typevar_operand(&dfg.value_lists));
            println!("  opcode: {:?}", instdata.opcode());
            let arguments = instdata.arguments(&dfg.value_lists);
            let types_of = |v: &[ir::Value]| v.iter().map(|z| dfg.value_type(*z)).collect::<Vec<_>>();
            println!("  args: {:?} types: {:?}", arguments, types_of(&arguments));
            
            println!("  results? {:?} -> {:?}  (types: {:?}) ",  dfg.has_results(inst), dfg.inst_results(inst), types_of(&dfg.inst_results(inst)));
            // We also don't have the types here... do WE have to propagate thos
        }
    }
    println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");

    // panic!();

    // let mut ctx = module.make_context();
    let mut ctx = cranelift_codegen::Context::for_function(fun);

    let name = "average";

    println!("Declaring function");
    let id = module
        .declare_function(name, Linkage::Export, &ctx.func.signature)
        .map_err(|e| e.to_string())?;

    println!("Defining function");
    module.define_function(id, &mut ctx).map_err(|e| {
        match &e {
            cranelift_module::ModuleError::Compilation(z) => match z {
                cranelift_codegen::CodegenError::Verifier(e) => {
                    for er in e.0.iter() {
                        println!("er: {er:?}");
                    }
                }
                _ => {}
            },
            _ => {}
        }
        e.to_string()
    })?;
    println!("fun_id: {id:?}");

    // Now that compilation is finished, we can clear out the context state.
    module.clear_context(&mut ctx);

    let finished_module = module.finish();
    let object_data = finished_module.emit()?;
    println!("object_data: {object_data:x?}");

    let write_to_disk = false;
    if write_to_disk {
        // Looks legit and is modified whenever the clif is modified.
        std::fs::write("/tmp/average.o", object_data)?;
    }

    Ok(())
}

fn main() -> ResultReturn {
    println!("Hello, world!");
    attempt_two()?;
    Ok(())
}
