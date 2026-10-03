use pspsdk::testrt::TestRunner;

mod condvar;
mod lazy_lock;
mod mutex;
mod once;
mod once_lock;
mod reentrant_lock;
mod rwlock;

#[macro_export]
macro_rules! nonpoison_and_poison_unwrap_test {
    (
        name: $name:ident,
        test_body: {$($test_body:tt)*}
    ) => {
        // Creates the nonpoison test.
        // #[test]
        pub fn ${concat(nonpoison_, $name)}() {
            #[allow(unused_imports)]
            use ::core::convert::identity as maybe_unwrap;
            use ::pspsdk::sync::nonpoison as locks;

            $($test_body)*
        }

        // Creates the poison test with the suffix `_unwrap_poisoned`.
        // #[test]
        pub fn ${concat(poison_, $name)}() {
            #[allow(unused_imports)]
            use super::result_unwrap as maybe_unwrap;
            use ::pspsdk::sync::poison as locks;

            $($test_body)*
        }
    }
}

#[track_caller]
fn result_unwrap<T, E: core::fmt::Debug>(x: Result<T, E>) -> T {
    x.unwrap()
}

pub fn test_group(tr: &mut TestRunner) {
    tr.test("condvar::poison_smoke", condvar::poison_smoke);
    tr.test("condvar::nonpoison_smoke", condvar::nonpoison_smoke);
    tr.test("condvar::poison_notify_one", condvar::poison_notify_one);
    tr.test("condvar::nonpoison_notify_one", condvar::nonpoison_notify_one);
    // tr.test("condvar::poison_notify_all", condvar::poison_notify_all);
    // tr.test("condvar::nonpoison_notify_all", condvar::nonpoison_notify_all);
    // tr.test("condvar::poison_test_mutex_arc_condvar", condvar::poison_test_mutex_arc_condvar);
    // tr.test("condvar::nonpoison_test_mutex_arc_condvar",
    // condvar::nonpoison_test_mutex_arc_condvar);
    tr.test("condvar::poison_wait_while", condvar::poison_wait_while);
    tr.test("condvar::nonpoison_wait_while", condvar::nonpoison_wait_while);
    tr.test("condvar::poison_wait_timeout_wait", condvar::poison_wait_timeout_wait);
    tr.test(
        "condvar::nonpoison_wait_timeout_wait",
        condvar::nonpoison_wait_timeout_wait,
    );
    tr.test(
        "condvar::poison_wait_timeout_while_wait",
        condvar::poison_wait_timeout_while_wait,
    );
    tr.test(
        "condvar::nonpoison_wait_timeout_while_wait",
        condvar::nonpoison_wait_timeout_while_wait,
    );
    tr.test(
        "condvar::poison_wait_timeout_while_instant_satisfy",
        condvar::poison_wait_timeout_while_instant_satisfy,
    );
    tr.test(
        "condvar::nonpoison_wait_timeout_while_instant_satisfy",
        condvar::nonpoison_wait_timeout_while_instant_satisfy,
    );
    tr.test(
        "condvar::poison_wait_timeout_while_wake",
        condvar::poison_wait_timeout_while_wake,
    );
    tr.test(
        "condvar::nonpoison_wait_timeout_while_wake",
        condvar::nonpoison_wait_timeout_while_wake,
    );
    tr.test("condvar::poison_wait_timeout_wake", condvar::poison_wait_timeout_wake);
    tr.test(
        "condvar::nonpoison_wait_timeout_wake",
        condvar::nonpoison_wait_timeout_wake,
    );
    tr.test("condvar::poison_timeout_nanoseconds", condvar::poison_timeout_nanoseconds);
    tr.test(
        "condvar::nonpoison_timeout_nanoseconds",
        condvar::nonpoison_timeout_nanoseconds,
    );
    // tr.test("condvar::test_arc_condvar_poison", condvar::test_arc_condvar_poison);


    tr.test("lazy_lock::lazy_default", lazy_lock::lazy_default);
    // tr.test("lazy_lock::const_lazy_default", lazy_lock::const_lazy_default);
    tr.test("lazy_lock::sync_lazy_new", lazy_lock::sync_lazy_new);
    tr.test("lazy_lock::sync_lazy_default", lazy_lock::sync_lazy_default);
    tr.test("lazy_lock::static_sync_lazy", lazy_lock::static_sync_lazy);
    tr.test("lazy_lock::static_sync_lazy_via_fn", lazy_lock::static_sync_lazy_via_fn);
    tr.test("lazy_lock::lazy_type_inference", lazy_lock::lazy_type_inference);
    tr.test("lazy_lock::is_sync_send", lazy_lock::is_sync_send);
    tr.test("lazy_lock::lazy_force_mut", lazy_lock::lazy_force_mut);
    tr.test("lazy_lock::lazy_poisoning", lazy_lock::lazy_poisoning);
    tr.should_panic("lazy_lock::lazy_lock_deref_panic", lazy_lock::lazy_lock_deref_panic);
    tr.should_panic(
        "lazy_lock::lazy_lock_deref_mut_panic",
        lazy_lock::lazy_lock_deref_mut_panic,
    );
    tr.should_panic(
        "lazy_lock::lazy_lock_preserves_closure_panic_message",
        lazy_lock::lazy_lock_preserves_closure_panic_message,
    );


    tr.test("mutex::smoke", mutex::smoke);
    // tr.test("mutex::lots_and_lots", mutex::lots_and_lots);
    tr.test("mutex::try_lock", mutex::try_lock);
    tr.test("mutex::test_into_inner", mutex::test_into_inner);
    tr.test("mutex::test_into_inner_drop", mutex::test_into_inner_drop);
    tr.test("mutex::test_get_cloned", mutex::test_get_cloned);
    tr.test("mutex::test_get_mut", mutex::test_get_mut);
    tr.test("mutex::test_set", mutex::test_set);
    tr.test("mutex::test_replace", mutex::test_replace);
    // tr.test("mutex::test_mutex_arc_condvar", mutex::test_mutex_arc_condvar);
    // tr.test("mutex::test_mutex_arc_nested", mutex::test_mutex_arc_nested);
    tr.test(
        "mutex::test_mutex_arc_access_in_unwind",
        mutex::test_mutex_arc_access_in_unwind,
    );
    tr.test("mutex::test_mutex_unsized", mutex::test_mutex_unsized);
    tr.test("mutex::test_mapping_mapped_guard", mutex::test_mapping_mapped_guard);
    tr.test("mutex::test_mutex_with_mut", mutex::test_mutex_with_mut);
    tr.test("mutex::test_needs_drop", mutex::test_needs_drop);
    tr.test("mutex::test_mutex_arc_poison", mutex::test_mutex_arc_poison);
    tr.test("mutex::test_mutex_arc_poison_mapped", mutex::test_mutex_arc_poison_mapped);
    tr.test(
        "mutex::panic_while_mapping_unlocked_poison",
        mutex::panic_while_mapping_unlocked_poison,
    );


    tr.test("once_lock::sync_once_cell", once_lock::sync_once_cell);
    tr.test("once_lock::sync_once_cell_get_mut", once_lock::sync_once_cell_get_mut);
    tr.test("once_lock::sync_once_cell_drop", once_lock::sync_once_cell_drop);
    tr.test(
        "once_lock::sync_once_cell_drop_empty",
        once_lock::sync_once_cell_drop_empty,
    );
    tr.test("once_lock::clone", once_lock::clone);
    tr.test("once_lock::get_or_try_init", once_lock::get_or_try_init);
    tr.test("once_lock::from_impl", once_lock::from_impl);
    tr.test("once_lock::partialeq_impl", once_lock::partialeq_impl);
    tr.test("once_lock::into_inner", once_lock::into_inner);
    tr.test("once_lock::is_sync_send", once_lock::is_sync_send);
    tr.test("once_lock::eval_once_macro", once_lock::eval_once_macro);
    // tr.test(
    //     "once_lock::sync_once_cell_does_not_leak_partially_constructed_boxes",
    //     once_lock::sync_once_cell_does_not_leak_partially_constructed_boxes,
    // );
    // tr.test("once_lock::dropck", once_lock::dropck);

    tr.test("once::smoke_once", once::smoke_once);
    // tr.test("once::stampede_once", once::stampede_once);
    tr.test("once::poison_bad", once::poison_bad);
    // tr.test("once::wait_for_force_to_finish", once::wait_for_force_to_finish);
    tr.test("once::wait", once::wait);
    tr.test("once::wait_on_poisoned", once::wait_on_poisoned);
    tr.test("once::wait_force_on_poisoned", once::wait_force_on_poisoned);

    tr.test("reentrant_lock::smoke", reentrant_lock::smoke);
    tr.test("reentrant_lock::is_mutex", reentrant_lock::is_mutex);
    tr.test("reentrant_lock::trylock_works", reentrant_lock::trylock_works);

    tr.test("rwlock::test_needs_drop", rwlock::test_needs_drop);
    tr.test("rwlock::poison_smoke", rwlock::poison_smoke);
    tr.test("rwlock::nonpoison_smoke", rwlock::nonpoison_smoke);
    // tr.test("rwlock::poison_frob", rwlock::poison_frob);
    // tr.test("rwlock::nonpoison_frob", rwlock::nonpoison_frob);
    // tr.test("rwlock::poison_test_rw_arc", rwlock::poison_test_rw_arc);
    // tr.test("rwlock::nonpoison_test_rw_arc", rwlock::nonpoison_test_rw_arc);
    tr.test(
        "rwlock::poison_test_rw_arc_access_in_unwind",
        rwlock::poison_test_rw_arc_access_in_unwind,
    );
    tr.test(
        "rwlock::nonpoison_test_rw_arc_access_in_unwind",
        rwlock::nonpoison_test_rw_arc_access_in_unwind,
    );
    tr.test("rwlock::poison_test_rwlock_unsized", rwlock::poison_test_rwlock_unsized);
    tr.test(
        "rwlock::nonpoison_test_rwlock_unsized",
        rwlock::nonpoison_test_rwlock_unsized,
    );
    tr.test("rwlock::poison_test_into_inner", rwlock::poison_test_into_inner);
    tr.test("rwlock::nonpoison_test_into_inner", rwlock::nonpoison_test_into_inner);
    tr.test("rwlock::poison_test_into_inner_drop", rwlock::poison_test_into_inner_drop);
    tr.test(
        "rwlock::nonpoison_test_into_inner_drop",
        rwlock::nonpoison_test_into_inner_drop,
    );
    tr.test("rwlock::poison_test_get_cloned", rwlock::poison_test_get_cloned);
    tr.test("rwlock::nonpoison_test_get_cloned", rwlock::nonpoison_test_get_cloned);
    tr.test("rwlock::poison_test_get_mut", rwlock::poison_test_get_mut);
    tr.test("rwlock::nonpoison_test_get_mut", rwlock::nonpoison_test_get_mut);
    tr.test("rwlock::poison_test_set", rwlock::poison_test_set);
    tr.test("rwlock::nonpoison_test_set", rwlock::nonpoison_test_set);
    tr.test("rwlock::poison_test_replace", rwlock::poison_test_replace);
    tr.test("rwlock::nonpoison_test_replace", rwlock::nonpoison_test_replace);
    tr.test(
        "rwlock::poison_test_read_guard_covariance",
        rwlock::poison_test_read_guard_covariance,
    );
    tr.test(
        "rwlock::nonpoison_test_read_guard_covariance",
        rwlock::nonpoison_test_read_guard_covariance,
    );
    tr.test(
        "rwlock::poison_test_mapped_read_guard_covariance",
        rwlock::poison_test_mapped_read_guard_covariance,
    );
    tr.test(
        "rwlock::nonpoison_test_mapped_read_guard_covariance",
        rwlock::nonpoison_test_mapped_read_guard_covariance,
    );
    tr.test("rwlock::poison_test_downgrade_basic", rwlock::poison_test_downgrade_basic);
    tr.test(
        "rwlock::nonpoison_test_downgrade_basic",
        rwlock::nonpoison_test_downgrade_basic,
    );
    tr.test(
        "rwlock::poison_test_downgrade_observe",
        rwlock::poison_test_downgrade_observe,
    );
    tr.test(
        "rwlock::nonpoison_test_downgrade_observe",
        rwlock::nonpoison_test_downgrade_observe,
    );
    tr.test(
        "rwlock::poison_test_downgrade_atomic",
        rwlock::poison_test_downgrade_atomic,
    );
    tr.test(
        "rwlock::nonpoison_test_downgrade_atomic",
        rwlock::nonpoison_test_downgrade_atomic,
    );
    tr.test(
        "rwlock::poison_test_mapping_mapped_guard",
        rwlock::poison_test_mapping_mapped_guard,
    );
    tr.test(
        "rwlock::nonpoison_test_mapping_mapped_guard",
        rwlock::nonpoison_test_mapping_mapped_guard,
    );
    tr.test(
        "rwlock::nonpoison_test_rwlock_try_write",
        rwlock::nonpoison_test_rwlock_try_write,
    );
    tr.test(
        "rwlock::poison_test_rwlock_try_write",
        rwlock::poison_test_rwlock_try_write,
    );
    tr.test("rwlock::test_into_inner_poison", rwlock::test_into_inner_poison);
    tr.test("rwlock::test_get_cloned_poison", rwlock::test_get_cloned_poison);
    tr.test("rwlock::test_get_mut_poison", rwlock::test_get_mut_poison);
    tr.test("rwlock::test_set_poison", rwlock::test_set_poison);
    tr.test("rwlock::test_replace_poison", rwlock::test_replace_poison);
    tr.test("rwlock::test_rw_arc_poison_wr", rwlock::test_rw_arc_poison_wr);
    tr.test(
        "rwlock::test_rw_arc_poison_mapped_w_r",
        rwlock::test_rw_arc_poison_mapped_w_r,
    );
    tr.test("rwlock::test_rw_arc_poison_ww", rwlock::test_rw_arc_poison_ww);
    tr.test(
        "rwlock::test_rw_arc_poison_mapped_w_w",
        rwlock::test_rw_arc_poison_mapped_w_w,
    );
    tr.test("rwlock::test_rw_arc_no_poison_rr", rwlock::test_rw_arc_no_poison_rr);
    tr.test(
        "rwlock::test_rw_arc_no_poison_mapped_r_r",
        rwlock::test_rw_arc_no_poison_mapped_r_r,
    );
    tr.test("rwlock::test_rw_arc_no_poison_rw", rwlock::test_rw_arc_no_poison_rw);
    tr.test(
        "rwlock::test_rw_arc_no_poison_mapped_r_w",
        rwlock::test_rw_arc_no_poison_mapped_r_w,
    );
    tr.test(
        "rwlock::panic_while_mapping_read_unlocked_no_poison",
        rwlock::panic_while_mapping_read_unlocked_no_poison,
    );
    tr.test(
        "rwlock::panic_while_mapping_write_unlocked_poison",
        rwlock::panic_while_mapping_write_unlocked_poison,
    );
    tr.test("rwlock::test_rwlock_with", rwlock::test_rwlock_with);
    tr.test("rwlock::test_rwlock_with_mut", rwlock::test_rwlock_with_mut);
    tr.test("rwlock::test_rwlock_max_readers", rwlock::test_rwlock_max_readers);
}
