#[test]
fn architecture_boundaries_are_enforced() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/private_work_service.rs");
    t.compile_fail("tests/ui/private_live_board.rs");
}
