use pspsdk::testrt::TestRunner;

pub mod mutex;

pub fn test_group(tr: &mut TestRunner) {
    tr.run_test("mutex::smoke", mutex::smoke);
    tr.run_test("mutex::test_needs_drop", mutex::test_needs_drop);
    tr.run_test("mutex::try_lock", mutex::try_lock);
    tr.run_test("mutex::test_into_inner", mutex::test_into_inner);
    tr.run_test("mutex::test_into_inner_drop", mutex::test_into_inner_drop);
    tr.run_test("mutex::test_get_cloned", mutex::test_get_cloned);
    tr.run_test("mutex::test_get_mut", mutex::test_get_mut);
    tr.run_test("mutex::test_set", mutex::test_set);
    tr.run_test("mutex::test_replace", mutex::test_replace);
    tr.run_test("mutex::test_mutex_unsized", mutex::test_mutex_unsized);
    tr.run_test("mutex::test_mapping_mapped_guard", mutex::test_mapping_mapped_guard);
    tr.run_test("mutex::test_mutex_with_mut", mutex::test_mutex_with_mut);
}
