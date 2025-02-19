pub use paste;

#[macro_export]
macro_rules! declare_double_lint {
    ($(#[$attr:meta])* $vis:vis $NAME:ident, $Level:ident, $desc:expr) => {
        $crate::paste::paste! {
            dylint_linting::dylint_library!();

            extern crate rustc_lint;
            extern crate rustc_session;

            #[no_mangle]
            #[allow(clippy::no_mangle_with_rust_abi)]
            pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
                dylint_linting::init_config(sess);
                lint_store.register_lints(&[$NAME]);
                lint_store.register_early_pass(|| Box::new([< $NAME:camel >]));
                lint_store.register_late_pass(|_| Box::new([< $NAME:camel >]));
            }

            rustc_session::declare_lint!($(#[$attr])* $vis $NAME, $Level, $desc);
            rustc_session::declare_lint_pass!([< $NAME:camel >] => [$NAME]);
        }
    };
}

#[macro_export]
macro_rules! declare_pre_expansion_and_late_lint {
    ($(#[$attr:meta])* $vis:vis $NAME:ident, $Level:ident, $desc:expr) => {
        $crate::paste::paste! {
            dylint_linting::dylint_library!();

            extern crate rustc_lint;
            extern crate rustc_session;

            #[no_mangle]
            #[allow(clippy::no_mangle_with_rust_abi)]
            pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
                dylint_linting::init_config(sess);
                lint_store.register_lints(&[$NAME]);
                lint_store.register_pre_expansion_pass(|| Box::new([< $NAME:camel >]));
                lint_store.register_late_pass(|_| Box::new([< $NAME:camel >]));
            }

            rustc_session::declare_lint!($(#[$attr])* $vis $NAME, $Level, $desc);
            rustc_session::declare_lint_pass!([< $NAME:camel >] => [$NAME]);
        }
    };
}
