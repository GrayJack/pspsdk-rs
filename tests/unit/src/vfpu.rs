use core::f32::consts::PI;

use pspsdk::testrt::TestRunner;

pub fn test_group(tr: &mut TestRunner) {
    tr.test("vfpu::test_s", test_s);
    tr.test("vfpu::test_q", test_q);
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
    Vec4 { x, y, z, w }
}

fn vec4_splat(num: f32) -> Vec4 {
    vec4(num, num, num, num)
}

macro_rules! vfpu_asm_return {
    ($($l:literal),*,$($i:ident$(.$e:expr)?),*,) => {
        unsafe {
            let mut o = ::core::mem::MaybeUninit::uninit();
            ::pspsdk::vfpu_asm!(
                $($l,)*
                in(reg) (&mut o),
                $(in(reg) (&$i$(.$e)*),)*
                options(nostack),
            );
            o.assume_init()
        }
    };
}

fn add_s(value: f32) -> f32 {
    let zero: f32 = 0.0;
    vfpu_asm_return!(
        "lv.s S010, {1}",
        "lv.s S020, {2}",
        "vadd.s S000, S010, S020",
        "sv.s S000, {0}",
        zero,
        value,
    )
}

fn sub_s(value: f32) -> f32 {
    let zero: f32 = 0.0;
    vfpu_asm_return!(
        "lv.s S010, {1}",
        "lv.s S020, {2}",
        "vsub.s S000, S010, S020",
        "sv.s S000, {0}",
        zero,
        value,
    )
}

fn mul_s(value: f32) -> f32 {
    let one: f32 = 1.0;
    vfpu_asm_return!(
        "lv.s S010, {1}",
        "lv.s S020, {2}",
        "vmul.s S000, S010, S020",
        "sv.s S000, {0}",
        one,
        value,
    )
}

fn div_s(value: f32) -> f32 {
    let one: f32 = 1.0;
    vfpu_asm_return!(
        "lv.s S010, {1}",
        "lv.s S020, {2}",
        "vdiv.s S000, S010, S020",
        "sv.s S000, {0}",
        one,
        value,
    )
}

fn dot(num: f32) -> f32 {
    let one = vec4_splat(1.0);
    let value = vec4_splat(num);
    vfpu_asm_return!(
        "lv.q C010, {1}",
        "lv.q C020, {2}",
        "vdot.q S000, C010, C020",
        "sv.s S000, {0}",
        one,
        value,
    )
}

fn test_s() {
    assert_eq!(add_s(0.0), 0.0);
    assert_eq!(add_s(1.0), 1.0);
    assert_eq!(sub_s(0.0), 0.0);
    assert_eq!(sub_s(1.0), -1.0);
    assert_eq!(mul_s(0.0), 0.0);
    assert_eq!(mul_s(2.0), 2.0);
    assert_eq!(div_s(1.0), 1.0);
    assert_eq!(div_s(2.0), 0.5);
    assert_eq!(dot(1.0), 4.0);
    assert_eq!(dot(2.0), 8.0);
}

fn add_q(num: f32) -> Vec4 {
    let zero = vec4_splat(0.0);
    let value = vec4_splat(num);
    vfpu_asm_return!(
        "lv.q C010, {1}",
        "lv.q C020, {2}",
        "vadd.q C000, C010, C020",
        "sv.q C000, {0}",
        zero,
        value,
    )
}

fn sub_q(num: f32) -> Vec4 {
    let zero = vec4_splat(0.0);
    let value = vec4_splat(num);
    vfpu_asm_return!(
        "lv.q C010, {1}",
        "lv.q C020, {2}",
        "vsub.q C000, C010, C020",
        "sv.q C000, {0}",
        zero,
        value,
    )
}

fn mul_q(num: f32) -> Vec4 {
    let one = vec4_splat(1.0);
    let value = vec4_splat(num);
    vfpu_asm_return!(
        "lv.q C010, {1}",
        "lv.q C020, {2}",
        "vmul.q C000, C010, C020",
        "sv.q C000, {0}",
        one,
        value,
    )
}

fn div_q(num: f32) -> Vec4 {
    let one = vec4_splat(1.0);
    let value = vec4_splat(num);
    vfpu_asm_return!(
        "lv.q C010, {1}",
        "lv.q C020, {2}",
        "vdiv.q C000, C010, C020",
        "sv.q C000, {0}",
        one,
        value,
    )
}

fn add_vcst_pi(num: f32) -> Vec4 {
    let value = vec4_splat(num);
    vfpu_asm_return!(
        "lv.q C010, {1}",
        "vcst.q C020, VFPU_PI",
        "vadd.q C000, C010, C020",
        "sv.q C000, {0}",
        value,
    )
}

fn shuffle_add_rev(value: Vec4) -> Vec4 {
    let zero = vec4_splat(0.0);
    vfpu_asm_return!(
        "lv.q C010, {1}",
        "lv.q C020, {2}",
        "vadd.q C000, C010, C020[W,Z,Y,X]",
        "sv.q C000, {0}",
        zero,
        value,
    )
}

fn shuffle_sub_rev(value: Vec4) -> Vec4 {
    let zero = vec4_splat(0.0);
    vfpu_asm_return!(
        "lv.q C010, {1}",
        "lv.q C020, {2}",
        "vadd.q C000, C010, C020[-W,-Z,-Y,-X]",
        "sv.q C000, {0}",
        zero,
        value,
    )
}

fn test_q() {
    assert_eq!(add_q(0.0), vec4_splat(0.0));
    assert_eq!(add_q(1.0), vec4_splat(1.0));
    assert_eq!(sub_q(0.0), vec4_splat(0.0));
    assert_eq!(sub_q(1.0), vec4_splat(-1.0));
    assert_eq!(mul_q(0.0), vec4_splat(0.0));
    assert_eq!(mul_q(2.0), vec4_splat(2.0));
    assert_eq!(div_q(1.0), vec4_splat(1.0));
    assert_eq!(div_q(2.0), vec4_splat(0.5));
    assert_eq!(add_vcst_pi(0.0), vec4_splat(PI),);
    assert_eq!(add_vcst_pi(1.0), vec4_splat(1.0 + PI),);
    assert_eq!(shuffle_add_rev(vec4(1.0, 2.0, 3.0, 4.0)), vec4(4.0, 3.0, 2.0, 1.0),);
    assert_eq!(shuffle_sub_rev(vec4(1.0, 2.0, 3.0, 4.0)), vec4(-4.0, -3.0, -2.0, -1.0),);
}
