#![feature(rustc_private)]
#![warn(unused_extern_crates)]

dylint_linting::dylint_library!();

extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod safe_path_lint;

#[doc(hidden)]
#[no_mangle]
pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_lints(&[safe_path_lint::SAFE_JOIN_OPPORTUNITY]);
    lint_store.register_lints(&[safe_path_lint::SAFE_JOIN_MISAPPLICATION]);
    lint_store.register_late_pass(|| Box::new(safe_path_lint::SafePathLint));
}

#[test]
fn ui_examples() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
