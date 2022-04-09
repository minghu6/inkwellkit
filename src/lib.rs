use either::Either;
pub use inkwell::*;

use inkwell::{
    module::{ Module, Linkage },
    context::{ Context, ContextRef }, basic_block::BasicBlock, builder::Builder, values::{PointerValue, IntValue, BasicMetadataValueEnum}
};

pub use proc_macros::{
    impl_fn_hdr,
    load_vm_common_ty
};


thread_local! {
    pub static CTX: ContextRef<'static> = ContextRef::new2();

    // pub static CTX: &'static Context = & Context::create();
}

#[inline]
pub fn get_ctx<'ctx>() -> &'ctx Context {
    CTX.with(|ctx| unsafe { ctx.get() })
}

pub struct VMMod<'ctx> {
    pub module: Module<'ctx>,
}

#[allow(unused)]
impl<'ctx> VMMod<'ctx> {
    pub fn new(name: &str) -> Self {
        let module = get_ctx().create_module(name);

        Self {
            module
        }
    }


    ///////////////////////////////////
    //// POSIX

    pub fn include_fcntl(&self) {
        let module = &self.module;

        impl_fn_hdr![ module |
            open(*i8, i32, ...) -> i32;
        ];
    }

    pub fn include_stdio(&self) {
        let module = &self.module;

        impl_fn_hdr![ module |
            printf(*i8, ...) -> i32;
        ];
    }

    pub fn include_string(&self) {
        let module = &self.module;

        impl_fn_hdr![ module |
            strlen(*i8) -> usize;
        ];
    }

    pub fn include_unistd(&self) {
        let module = &self.module;

        impl_fn_hdr![ module |
            write(i32, *i8, usize) -> i128;
            close(i32) -> i32;
            sleep(u32) -> u32;
        ];
    }


    ///////////////////////////////////
    //// POSIX
    pub fn get_builder() -> Builder<'ctx> {
        get_ctx().create_builder()
    }

    pub fn get_builder_at_end(
        blk: BasicBlock<'ctx>,
    ) -> Builder<'ctx> {
        let builder = Self::get_builder();

        builder.position_at_end(blk);

        builder
    }

    pub fn get_builder_at_start(
        blk: BasicBlock<'ctx>,
    ) -> Builder<'ctx> {
        let builder = Self::get_builder();

        builder_position_at_start(&builder, blk);

        builder
    }

    pub fn append_main(&self, module: &Module<'ctx>) -> BasicBlock<'ctx> {
        load_vm_common_ty!(get_ctx());

        let fn_main_t = i64_t.fn_type(&[], false);
        let fn_main = module.add_function("main", fn_main_t, None);

        get_ctx().append_basic_block(fn_main, "blk_main")
    }


    //////////////////////////////////////////////////////////////////////
    //// Convenient Build
    //////////////////////////////////////////////////////////////////////

    pub fn bload_int(
        &self,
        builder: &Builder<'ctx>,
        var: PointerValue<'ctx>,
    ) -> IntValue<'ctx> {
        builder.build_load(var, "").into_int_value()
    }

    pub fn bcnt_init(
        &self,
        builder: &Builder<'ctx>,
        init: IntValue<'ctx>,
    ) -> PointerValue<'ctx> {
        load_vm_common_ty!(get_ctx());

        let var = builder.build_alloca(i32_t, "");
        builder.build_store(var, init);

        var
    }

    pub fn bcnt_forward(
        &self,
        builder: &Builder<'ctx>,
        var: PointerValue<'ctx>,
        step: IntValue<'ctx>
    )
    {
        let val = self.bload_int(builder, var);
        let nxt = builder.build_int_add(val, step, "");
        builder.build_store(var, nxt);
    }

    /// (low, high)
    pub fn bcnt_check(
        &self,
        builder: &Builder<'ctx>,
        var: PointerValue<'ctx>,
        test: Either<IntValue<'ctx>, IntValue<'ctx>>
    ) -> IntValue<'ctx>
    {
        let val = self.bload_int(builder, var);

        match test {
            Either::Left(low) => {
                self.bsgt(builder, val, low)
            },
            Either::Right(high) => {
                self.bsgt(builder, high, val)
            },
        }
    }

    pub fn build_local_str(
        &self,
        builder: &Builder<'ctx>,
        value: &str,
    ) -> (PointerValue<'ctx>, IntValue<'ctx>) {
        load_vm_common_ty!(get_ctx());
        let var = get_ctx().const_string(value.as_bytes(), true);
        let len = self.usize(value.len());

        let var_ptr = builder.build_alloca(var.get_type(), "");
        builder.build_store(var_ptr, var);

        let var_ptr_cast = builder
            .build_bitcast(var_ptr, i8ptr_t, "")
            .into_pointer_value();

        (var_ptr_cast, len)
    }

    /// (*u8, len)
    pub fn build_local_const_u8_array(
        &self,
        builder: &Builder<'ctx>,
        values: &[IntValue<'ctx>],
    ) -> (PointerValue<'ctx>, IntValue<'ctx>) {
        load_vm_common_ty!(get_ctx());

        let var= i8_t.const_array(values);
        let len = self.usize((values.len() as u64).try_into().unwrap());

        let var_ptr = builder.build_alloca(
            var.get_type(),
            ""
        );
        builder.build_store(var_ptr, var);

        let var_ptr_cast = builder
            .build_bitcast(var_ptr, i8ptr_t, "")
            .into_pointer_value();

        (var_ptr_cast, len)
    }

    /// (*usize, len)
    pub fn build_local_const_usize_array(
        &self,
        builder: &Builder<'ctx>,
        values: &[IntValue<'ctx>],
    ) -> (PointerValue<'ctx>, IntValue<'ctx>) {
        load_vm_common_ty!(get_ctx());

        let var= size_t.const_array(values);
        let len = self.usize((values.len() as u64).try_into().unwrap());

        let var_ptr = builder.build_alloca(
            var.get_type(),
            ""
        );
        builder.build_store(var_ptr, var);

        let var_ptr_cast = builder
            .build_bitcast(var_ptr, sizeptr_t, "")
            .into_pointer_value();

        (var_ptr_cast, len)
    }

    /// (*usize, len)
    pub fn build_local_dyn_usize_array(
        &self,
        builder: &Builder<'ctx>,
        values: &[IntValue<'ctx>],
    ) -> (PointerValue<'ctx>, IntValue<'ctx>) {
        load_vm_common_ty!(get_ctx());

        let len = self.usize((values.len() as u64).try_into().unwrap());

        let var_ptr = builder.build_array_alloca(
            size_t,
            len,
            ""
        );
        for (i, value) in values.into_iter().enumerate() {
            let idx = self.usize(i);
            let ptr = unsafe {
                builder.build_in_bounds_gep(var_ptr, &[idx], "")
            };
            builder.build_store::<IntValue>(ptr, (*value).into());
        }

        let var_ptr_cast = builder
            .build_bitcast(var_ptr, sizeptr_t, "")
            .into_pointer_value();

        (var_ptr_cast, len)
    }

    pub fn build_call_printf(
        &self,
        builder: &Builder<'ctx>,
        module: &Module<'ctx>,
        fcs: &str,
        values: &[BasicMetadataValueEnum<'ctx>],
    ) {
        let (fcs_p, _) = self.build_local_str(builder, fcs);
        let fn_printf = module.get_function("printf").unwrap();

        let mut args = vec![fcs_p.into()];
        args.extend_from_slice(values);

        builder.build_call(fn_printf, &args[..], "");
    }


    //////////////////////////////////////////////////////////////////////
    //// Convenient Const
    //////////////////////////////////////////////////////////////////////

    // i8_t
    pub fn u8(&self, value: u8) -> IntValue<'ctx> {
        load_vm_common_ty!(get_ctx());

        i8_t.const_int(value as u64, false)
    }

    pub fn i32(&self, value: i32) -> IntValue<'ctx> {
        load_vm_common_ty!(get_ctx());

        i32_t.const_int(value as u64, false)
    }

    // size_t
    pub fn usize(&self, value: usize) -> IntValue<'ctx> {
        load_vm_common_ty!(get_ctx());

        size_t.const_int(value as u64, false)
    }


    //////////////////////////////////////////////////////////////////////
    //// Convenient Cmp
    //////////////////////////////////////////////////////////////////////

    pub fn bsgt(
        &self,
        builder: &Builder<'ctx>,
        x: IntValue<'ctx>,
        y: IntValue<'ctx>,
    ) -> IntValue<'ctx> {
        builder.build_int_compare(IntPredicate::SGT, x, y, "")
    }

    pub fn bsge(
        &self,
        builder: &Builder<'ctx>,
        x: IntValue<'ctx>,
        y: IntValue<'ctx>,
    ) -> IntValue<'ctx> {
        builder.build_int_compare(IntPredicate::SGE, x, y, "")
    }
}

pub fn builder_position_at_start<'ctx>(
    builder: &Builder<'ctx>,
    entry: BasicBlock<'ctx>,
) {
    match entry.get_first_instruction() {
        Some(first_instr) => builder.position_before(&first_instr),
        None => builder.position_at_end(entry),
    }
}

#[cfg(test)]
mod tests {

}
