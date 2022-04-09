use std::error::Error;

use inkwellkit::{
    VMMod,
    get_ctx, load_vm_common_ty,
    AddressSpace
};


fn test_io() -> Result<(), Box<dyn Error>> {
    let vmmod = VMMod::new("posix_llvm2");

    vmmod.include_stdio();
    vmmod.include_unistd();
    vmmod.include_fcntl();

    let builder = VMMod::get_builder();
    load_vm_common_ty!(get_ctx());

    // begin main
    let blk_main = holder.append_main(&vmmod);
    builder.position_at_end(blk_main);

    // open it
    let fn_open = vmmod.get_function("open").unwrap();
    let (fns, _) = holder.build_local_str(&builder, "hello.txt");
    let flags = i32_t.const_int(
        (libc::O_CREAT | libc::O_WRONLY | libc::O_APPEND)
            .try_into()
            .unwrap(),
        true,
    );
    let mode = i32_t.const_int(0o6666, false);

    let fd = ret_as_bv!(builder.build_call(
        fn_open,
        &[fns.into(), flags.into(), mode.into()],
        ""
    ));

    // write to it
    let fn_write = vmmod.get_function("write").unwrap();
    let (content, content_len) =
        holder.build_local_str(&builder, "_____写入了");
    let write_ok = ret_as_bv!(builder.build_call(
        fn_write,
        &[fd.into(), content.into(), content_len.into()],
        "",
    ));

    holder.build_call_printf(
        &builder,
        &vmmod,
        "write res: %d\n",
        &[write_ok.into()],
    );

    let fn_main = vmmod.get_function("main").unwrap();

    // // TEST IF ELSE
    // // compare int res > 5
    // let write_ok_cast =
    //     builder.build_int_truncate(write_ok.into_int_value(), i32_t, "");
    // let if_cond =
    //     holder.bsgt(&builder, write_ok_cast, holder.i32(123));
    // // let (then_builder, builder) = holder.bif(&builder, if_cond, fn_main);

    // let (then_builder, else_builder, builder) =
    //     holder.bif_else(&builder, if_cond, fn_main);

    // holder.build_call_printf(
    //     &then_builder,
    //     &module,
    //     " Enter Then: write res: %d\n",
    //     &[write_ok.into()],
    // );
    // holder.build_call_printf(
    //     &else_builder,
    //     &module,
    //     " Enter Else: write res: %d\n",
    //     &[write_ok.into()],
    // );

    // close it


    // end main
    builder.build_return(Some(&i64_t.const_zero()));
    fn_main.verify(true);

    print_obj(&vmmod, OptimizationLevel::None)?;
    println!("->: {}", module_name!());
    let output = module_name!() + ".out";
    link2_default(&output)?;
    run_bin(&output)
}



fn main() {
    println!("Hello, world!");
}
