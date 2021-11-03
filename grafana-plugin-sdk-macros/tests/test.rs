#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui-pass/basic.rs");
    t.pass("tests/ui-pass/init_subscriber.rs");
    t.pass("tests/ui-pass/shutdown_handler.rs");
    t.compile_fail("tests/ui-fail/missing_services.rs");
    t.compile_fail("tests/ui-fail/duplicate_services.rs");
    t.compile_fail("tests/ui-fail/non_impl_service.rs");
    t.compile_fail("tests/ui-fail/invalid_init_subscriber.rs");
    t.compile_fail("tests/ui-fail/invalid_shutdown_handler.rs");
}
