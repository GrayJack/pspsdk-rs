use pspsdk::testrt::TestRunner;

pub mod mutex;

pub fn test_group(tr: &mut TestRunner) {
    tr.test("mutex::smoke", mutex::smoke);
    tr.test("mutex::test_needs_drop", mutex::test_needs_drop);
    tr.test("mutex::try_lock", mutex::try_lock);
    tr.test("mutex::test_into_inner", mutex::test_into_inner);
    tr.test("mutex::test_into_inner_drop", mutex::test_into_inner_drop);
    tr.test("mutex::test_get_cloned", mutex::test_get_cloned);
    tr.test("mutex::test_get_mut", mutex::test_get_mut);
    tr.test("mutex::test_set", mutex::test_set);
    tr.test("mutex::test_replace", mutex::test_replace);
    tr.test("mutex::test_mutex_unsized", mutex::test_mutex_unsized);
    tr.test("mutex::test_mapping_mapped_guard", mutex::test_mapping_mapped_guard);
    tr.test("mutex::test_mutex_with_mut", mutex::test_mutex_with_mut);
}
