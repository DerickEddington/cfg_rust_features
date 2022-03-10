pub(crate) fn emit_cargo_instruction(
    instruction: &str,
    arg: Option<&str>,
)
{
    assert!(!instruction.is_empty());
    if let Some(arg) = arg {
        assert!(!arg.is_empty());
    }
    println!("cargo:{}{}", instruction, arg.map(|s| format!("={}", s)).as_deref().unwrap_or(""));
}

/// Tell Cargo to display the given warning message after a build script has finished running.
pub fn emit_warning(message: &str)
{
    emit_cargo_instruction("warning", Some(message));
}

/// Pass a key-value configuration option to the compiler to be set for conditional compilation,
/// for features of the Rust compiler, language, or standard library.
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
pub(crate) fn emit_rust_feature(
    category: &str,
    name: &str,
)
{
    assert!(["comp", "lang", "lib"].contains(&category));
    emit_cargo_instruction("rustc-cfg", Some(&format!("rust_{}_feature={:?}", category, name)));
}
