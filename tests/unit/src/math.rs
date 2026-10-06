use core::f32::consts::PI;

use pspsdk::{math, testrt::TestRunner};

pub fn test_group(tr: &mut TestRunner) {
    tr.test("math::test_cos", test_cos);
    tr.test("math::test_sin", test_sin);
    tr.test("math::test_fmodf", test_fmodf);
    tr.test("math::test_fminf", test_fminf);
    tr.test("math::test_fmaxf", test_fmaxf);

    tr.bench("math::cos", test_cos, 100);
    tr.bench("math::sin", test_sin, 100);
    tr.bench("math::fmodf", test_fmodf, 100);
}

fn test_cos() {
    assert_eq!(math::cosf(0.0), 1.0);
    assert_eq!(math::cosf(PI), -1.0);

    let cos_2_5 = math::cosf(2.5) + 0.8011436;
    assert!(cos_2_5 < (f32::EPSILON * 2.0) && cos_2_5 > -(f32::EPSILON * 2.0));
}

fn test_sin() {
    assert_eq!(math::sinf(0.0), 0.0);
    assert_eq!(math::sinf(2.5), 0.5984721);

    let almost_zero = math::sinf(PI);
    assert!(almost_zero < f32::EPSILON && almost_zero > -f32::EPSILON);
}

fn test_fminf() {
    assert_eq!(math::fminf(-10.0, 3.0), -10.0);
    assert_eq!(math::fminf(-10.0, f32::NAN), -10.0);
}

fn test_fmaxf() {
    assert_eq!(math::fmaxf(-10.0, 3.0), 3.0);
    assert_eq!(math::fmaxf(3.0, f32::NAN), 3.0);
}

fn test_fmodf() {
    assert_eq!(math::fmodf(-10.0, 3.0), -1.0);
    let fmodf_0 = math::fmodf(1.0, 0.0);
    assert!(fmodf_0.is_nan());
}
