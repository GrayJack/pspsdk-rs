#[allow(unused)]
unsafe extern "C" {
    /// Call a function with the signature `fn(i32, i64, i32) -> i64` via the MIPS-EABI ABI.
    ///
    /// This is not safe to call with a function that expects any other ABI.
    pub fn i_ii_i_rii(
        a: u32, b: u64, c: u32, ptr: unsafe extern "C" fn(u32, u64, u32) -> u64,
    ) -> u64;

    /// Call a function with the signature `fn(i32, i64, i32) -> i32` via the MIPS-EABI ABI.
    ///
    /// This is not safe to call with a function that expects any other ABI.
    pub fn i_ii_i_ri(
        a: u32, b: u64, c: u32, ptr: unsafe extern "C" fn(u32, u64, u32) -> u32,
    ) -> u32;
}

// Potential resource:
// (scroll down to table) https://www.linux-mips.org/wiki/P32_Linux_ABI
//
// Page 3-18: http://web.archive.org/web/20040930224745/http://www.caldera.com/developers/devspecs/mipsabi.pdf
// Copied from PDF:
// Despite the fact that some or all of the arguments to a function are passed
// in registers, always allocate space on the stack for all arguments. This
// stack space should be a structure large enough to contain all the arguments,
// aligned according to normal structure rules (after promotion and structure
// return pointer insertion). The locations within the stack frame used for
// arguments are called the home locations.
#[cfg(target_os = "psp")]
core::arch::global_asm!(
    r#"
        .global i_ii_i_rii
        .global i_ii_i_ri
        i_ii_i_rii:
        i_ii_i_ri:
            addiu $sp, -32
            sw $ra, 8($sp)

            lw $t0, 48($sp)
            lw $t1, 52($sp)
            jalr $t1

            lw $ra, 8($sp)
            addiu $sp, 32
            jr $ra
    "#
);
