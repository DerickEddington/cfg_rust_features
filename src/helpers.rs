/// Print to `stdout` a build-script instruction for Cargo.
///
/// # Panics
/// If either argument is an empty string.
///
/// (Actually private to the crate, not part of public API.  Is only `pub` for old Rust versions.)
pub fn emit_cargo_instruction(
    instruction: &str,
    arg: Option<&str>,
)
{
    assert!(!instruction.is_empty());
    if let Some(arg) = arg {
        assert!(!arg.is_empty());
    }
    println!(
        "cargo:{}{}",
        instruction,
        arg.map(|s| format!("={}", s)).unwrap_or_else(String::new)
    );
}

/// Tell Cargo to display the given warning message after a build script has finished running.
pub fn emit_warning(message: &str)
{
    emit_cargo_instruction("warning", Some(message));
}

/// Tell Cargo to pass a key-value configuration option to the compiler to be set for conditional
/// compilation, for features of the Rust compiler, language, or standard library.
///
/// This enables using [the standard conditional-compilation
/// forms](https://doc.rust-lang.org/reference/conditional-compilation.html) (i.e. the `cfg`
/// attribute, et al) for features of Rust itself, in a way that is more similar to Cargo package
/// features.
///
/// `category`: One of `"comp"`, `"lang"`, or `"lib"`.
///
/// `value`: The feature name, which should follow [The Unstable
/// Book](https://doc.rust-lang.org/nightly/unstable-book/index.html) where appropriate.
///
/// # Examples
///
/// Doing `emit_rust_feature("lib", "step_trait")` in a package's build script enables the
/// package's source code to use `#[cfg(rust_lib_feature = "step_trait")]`.
///
/// # Panics
///
/// If `category` is not one of the acceptable categories.
///
/// (Actually private to the crate, not part of public API.  Is only `pub` for old Rust versions.)
pub fn emit_rust_feature(
    category: &str,
    name: &str,
)
{
    assert!(["comp", "lang", "lib"].contains(&category));
    emit_cargo_instruction("rustc-cfg", Some(&format!("rust_{}_feature={:?}", category, name)));
}
