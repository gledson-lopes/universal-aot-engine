use cranelift::prelude::*;
use cranelift_codegen::Context;
// ✅ Use MemFlagsData directly instead of MemFlags
use cranelift::codegen::ir::MemFlagsData;
use cranelift_codegen::ir::immediates::Offset32;
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::create_dir_all(&out_dir).unwrap();

    let isa_builder = cranelift_native::builder().expect("host machine not supported");
    let isa = isa_builder
        .finish(settings::Flags::new(settings::builder()))
        .unwrap();

    let target_config = isa.frontend_config();

    let obj_builder = ObjectBuilder::new(
        isa,
        "univ_engine",
        cranelift_module::default_libcall_names(),
    )
    .unwrap();
    let mut module = ObjectModule::new(obj_builder);
    let mut b_ctx = FunctionBuilderContext::new();

    // --- FUNCTION: rt_get_tag(ptr: i64) -> i32 ---
    let mut sig_tag = module.make_signature();
    sig_tag.params.push(AbiParam::new(types::I64));
    sig_tag.returns.push(AbiParam::new(types::I32));
    let func_tag = module
        .declare_function("rt_get_tag", Linkage::Export, &sig_tag)
        .unwrap();

    let mut ctx_tag = Context::new();
    ctx_tag.func.signature = sig_tag;
    {
        let mut builder = FunctionBuilder::new(&mut ctx_tag.func, &mut b_ctx);
        let block = builder.create_block();
        builder.append_block_params_for_function_params(block);
        builder.switch_to_block(block);

        let ptr = builder.block_params(block)[0];

        // ✅ FIX 1: Used MemFlagsData::new()
        let tag = builder.ins().load(types::I32, MemFlagsData::new(), ptr, 0);
        builder.ins().return_(&[tag]);
        builder.seal_block(block);
        builder.finalize(target_config);
    }
    module.define_function(func_tag, &mut ctx_tag).unwrap();

    // --- FUNCTION: rt_get_payload(ptr: i64, offset: i32) -> i32 ---
    let mut sig_pay = module.make_signature();
    sig_pay.params.push(AbiParam::new(types::I64));
    sig_pay.params.push(AbiParam::new(types::I32));
    sig_pay.returns.push(AbiParam::new(types::I32));
    let func_pay = module
        .declare_function("rt_get_payload", Linkage::Export, &sig_pay)
        .unwrap();

    let mut ctx_pay = Context::new();
    ctx_pay.func.signature = sig_pay;
    {
        let mut builder = FunctionBuilder::new(&mut ctx_pay.func, &mut b_ctx);
        let block = builder.create_block();
        builder.append_block_params_for_function_params(block);
        builder.switch_to_block(block);

        let params = builder.block_params(block);
        let ptr = params[0];
        let offset = params[1];

        let payload_base = builder.ins().iadd_imm_s(ptr, 4);
        let off64 = builder.ins().uextend(types::I64, offset);
        let addr = builder.ins().iadd(payload_base, off64);

        // ✅ FIX 2: Used MemFlagsData::new() (which maps seamlessly into defaults)
        let val = builder
            .ins()
            .load(types::I32, MemFlagsData::new(), addr, Offset32::new(0));

        builder.ins().return_(&[val]);
        builder.seal_block(block);
        builder.finalize(target_config);
    }
    module.define_function(func_pay, &mut ctx_pay).unwrap();

    let product = module.finish();
    let object_bytes = product.object.write().unwrap();

    let obj_path = out_dir.join("univ_engine.o");
    File::create(&obj_path)
        .unwrap()
        .write_all(&object_bytes)
        .unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-arg={}", obj_path.display());
}
