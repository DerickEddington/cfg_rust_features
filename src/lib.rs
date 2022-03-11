/*!
TODO

# Motivation

TODO: Convenient for having cond-comp that can detect when previously-unstable features become
stabilized (e.g. to adjust to use a new feature instead of previous workarounds).

TODO: More convenient than autocfg or version_check, when my goals are desired

# Examples

TODO: plural

# Minimum Supported Rust Version

Rust 1.0.0 will always be supported, so this crate can be used by other crates which support that
old version.

# Documentation

The source-code has doc comments, which are rendered as the API documentation.

View online at: <http://docs.rs/cfg_rust_features>

Or, you can generate them yourself and view locally by doing:

```shell
cargo doc --open
```
 */
// Remember to run `cargo readme` when changing the above doc-comment.

#![forbid(unsafe_code)]
#![allow(unknown_lints, deprecated, bare_trait_objects)]
// Warn about desired lints that would otherwise be allowed by default.
#![warn(
    // Groups
    future_incompatible,
    nonstandard_style,
    unused,
    // Individual lints not included in above groups and desired.
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    // missing_doc_code_examples, // maybe someday
    noop_method_call,
    pointer_structural_match,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences,
)]
// Warn about this one but avoid annoying hits for dev-dependencies.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]


extern crate autocfg;
extern crate version_check;

mod errors;
mod helpers;

pub use errors::UnsupportedFeatureTodoError;
pub use helpers::emit_warning;

use errors::VersionCheckError;
use std::collections::HashMap;
use std::error::Error;


/// Name of a feature, as recognized by this crate.
pub type FeatureName<'l> = &'l str;
/// Name of a feature category, as defined by this crate.
pub type FeatureCategory = &'static str;
/// Whether a feature is enabled and its category if so.
pub type FeatureEnabled = Option<FeatureCategory>;
/// Indicates whether each from a set of features was found to be enabled and its category.
pub type EnabledFeatures<'l> = HashMap<FeatureName<'l>, FeatureEnabled>;

/// Rust 1.0.0 does not support the `dyn` keyword.  This helps be clearer.
pub type ResultDynErr<T> = Result<T, Box<Error>>;


/// Tell Cargo to not default to scanning the entire package directory for changes, but to check
/// only given files, when deciding if a build script needs to be rerun.
///
/// Intended to be called from a package's build script.
pub fn emit_rerun_if_changed_file(filename: &str)
{
    helpers::emit_cargo_instruction("rerun-if-changed", Some(filename));
}


/// Information about the current Rust compiler.
///
/// Gathered when a [new intance is created](CfgRustFeatures::new).  Used to emit
/// [conditional-compilation configuration-options for use with the `cfg` et al
/// attributes](https://doc.rust-lang.org/reference/conditional-compilation.html).
///
/// Intended to be used from a package's build script.
#[derive(Debug)]
pub struct CfgRustFeatures
{
    /// Result of a run of the [`autocfg`] crate's information gathering.
    autocfg:       autocfg::AutoCfg,
    /// Result of a run of the [`version_check`] crate's information gathering.
    version_check: VersionCheck,
}

#[derive(Debug)]
struct VersionCheck
{
    #[allow(dead_code)]
    version: version_check::Version,
    channel: version_check::Channel,
    #[allow(dead_code)]
    date:    version_check::Date,
}

impl CfgRustFeatures
{
    /// Gather the information about the current Rust compiler, and return a new instance that can
    /// perform the operations with it.
    ///
    /// Intended to be called from a package's build script.
    ///
    /// # Errors
    /// If the information gathering fails.  (E.g., if the `OUT_DIR` environment variable is not
    /// set, or if `rustc` could not be run.)
    pub fn new() -> ResultDynErr<Self>
    {
        Self::with_autocfg(try!(autocfg::AutoCfg::new()))
    }

    fn with_autocfg(autocfg: autocfg::AutoCfg) -> ResultDynErr<Self>
    {
        if let Some((version, channel, date)) = version_check::triple() {
            Ok(CfgRustFeatures {
                autocfg: autocfg,
                version_check: VersionCheck { version: version, channel: channel, date: date }
            })
        }
        else {
            Err(VersionCheckError.into())
        }
    }

    /// Write, to `stdout`, instructions for Cargo to set configuration options that indicate
    /// whether the currently-used version of Rust (compiler, language, and library) supports the
    /// given sequence of feature names.
    ///
    /// Intended to be called from a package's build script.
    ///
    /// The supported feature names are particular to this crate but do correspond to [The
    /// Unstable Book](https://doc.rust-lang.org/nightly/unstable-book/index.html) where
    /// appropriate, but there are some extra feature names, like `"unstable_features"`, that are
    /// also supported.
    ///
    /// Each feature's configuration-option identifier has a naming scheme that categorizes
    /// the feature according to whether it pertains to the compiler (`rust_comp_feature`), the
    /// language (`rust_lang_feature`), or the standard library (`rust_lib_feature`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate cfg_rust_features;
    /// # extern crate create_temp_subdir;
    /// # use cfg_rust_features::{CfgRustFeatures, ResultDynErr};
    /// # use create_temp_subdir::TempSubDir;
    /// #
    /// # fn main() {
    /// #     let dir = TempSubDir::new("doctest-emit_rust_features").unwrap();
    /// #     std::env::set_var("OUT_DIR", &dir);
    /// #
    /// #     fn make_try_work() -> ResultDynErr<()> {
    /// let gathered_info_instance = try!(CfgRustFeatures::new());
    /// let enabled_features = try!(gathered_info_instance.emit_rust_features(vec![
    ///     "cfg_version",
    ///     "destructuring_assignment",
    ///     "inner_deref",
    ///     "iter_zip",
    ///     "never_type",
    ///     "question_mark",
    ///     "step_trait",
    ///     "unwrap_infallible",
    ///     "unstable_features",
    /// ]));
    /// #         Ok(())
    /// #     }
    /// #     make_try_work().unwrap();
    /// # }
    /// ```
    ///
    /// with `rustc` version `1.0`, will write nothing to `stdout`.
    ///
    /// or, with `rustc` version `1.56`, will write to `stdout`:
    /// ```text
    /// cargo:rustc-cfg=rust_lang_feature="question_mark"
    /// cargo:rustc-cfg=rust_lib_feature="inner_deref"
    /// ```
    ///
    /// or, with `rustc` version `1.59`, will write to `stdout`:
    /// ```text
    /// cargo:rustc-cfg=rust_lang_feature="destructuring_assignment"
    /// cargo:rustc-cfg=rust_lang_feature="question_mark"
    /// cargo:rustc-cfg=rust_lib_feature="iter_zip"
    /// cargo:rustc-cfg=rust_lib_feature="inner_deref"
    /// ```
    ///
    /// or, with `rustc` version `1.61.0-nightly`, will write to `stdout`:
    /// ```text
    /// cargo:rustc-cfg=rust_comp_feature="unstable_features"
    /// cargo:rustc-cfg=rust_lang_feature="destructuring_assignment"
    /// cargo:rustc-cfg=rust_lang_feature="question_mark"
    /// cargo:rustc-cfg=rust_lib_feature="inner_deref"
    /// cargo:rustc-cfg=rust_lib_feature="iter_zip"
    /// ```
    ///
    /// # Returns
    ///
    /// A [`HashMap`] that indicates whether each of the given features was found to be enabled
    /// and its category if so.  May be ignored, since the instructions for Cargo are also written
    /// out.
    ///
    /// # Errors
    ///
    /// If a feature name is unsupported by this crate currently.  The message will show the URL
    /// where a new issue may be opened to request adding support for the feature.
    pub fn emit_rust_features<'l, I: IntoIterator<Item = FeatureName<'l>>>(
        &self,
        features_names: I,
    ) -> Result<EnabledFeatures<'l>, UnsupportedFeatureTodoError>
    {
        use std::iter::repeat;

        let mut features_enabled: HashMap<_, _> =
            features_names.into_iter().zip(repeat(None)).collect();
        let mut any_stable_rust_feature = false;

        for (feature_name, enabled) in &mut features_enabled {
            *enabled = try!(self.emit_rust_feature(feature_name));
            any_stable_rust_feature = enabled.is_some() || any_stable_rust_feature;
        }
        if any_stable_rust_feature && try!(self.probe_rust_feature("cfg_version")).is_some() {
            emit_warning("Rust feature cfg_version is now stable. Consider using instead.");
        }
        Ok(features_enabled)
    }

    fn emit_rust_feature<'l>(
        &self,
        feature_name: FeatureName<'l>,
    ) -> Result<FeatureEnabled, UnsupportedFeatureTodoError>
    {
        self.probe_rust_feature(feature_name).map(|enabled| {
            enabled.map(|category| {
                helpers::emit_rust_feature(category, feature_name);
                category
            })
        })
    }

    /// Tests whether the current `rustc` provides the given compiler/language/library feature as
    /// stable (i.e. without needing the `#![feature(...)]` of nightly).
    ///
    /// # Returns
    /// The category of the feature if the feature is enabled, or else `None`.
    ///
    /// # Errors
    /// If the feature name is unsupported by this crate currently.
    fn probe_rust_feature<'l>(
        &self,
        feature_name: FeatureName<'l>,
    ) -> Result<FeatureEnabled, UnsupportedFeatureTodoError>
    {
        // TODO: Could improve with some static CATEGORY_TABLE: Once that associates feature to
        // category, which would allow factoring-out redundant `const CATEGORY` and redundant
        // `.then(|| ...)`.

        match feature_name {
            "cfg_version" => {
                const CATEGORY: &'static str = "lang";
                const EXPR: &'static str = r#"{ #[cfg(version("1.0"))] struct X; X }"#;
                Ok(if self.autocfg.probe_expression(EXPR) { Some(CATEGORY) } else { None })
            },
            "destructuring_assignment" => {
                const CATEGORY: &'static str = "lang";
                const EXPR: &'static str = r#"{ let (_a, _b); (_a, _b) = (1, 2); }"#;
                Ok(if self.autocfg.probe_expression(EXPR) { Some(CATEGORY) } else { None })
            },
            "inner_deref" => {
                const CATEGORY: &'static str = "lib";
                const EXPR: &'static str = r#"Ok::<_, ()>(vec![1]).as_deref()"#;
                Ok(if self.autocfg.probe_expression(EXPR) { Some(CATEGORY) } else { None })
            },
            "iter_zip" => {
                const CATEGORY: &'static str = "lib";
                const EXPR: &'static str = r#"std::iter::zip([1], ['a'])"#;
                Ok(if self.autocfg.probe_expression(EXPR) { Some(CATEGORY) } else { None })
            },
            "never_type" => {
                const CATEGORY: &'static str = "lang";
                const TYPE: &'static str = "!";
                Ok(if self.autocfg.probe_type(TYPE) { Some(CATEGORY) } else { None })
            },
            "question_mark" => {
                const CATEGORY: &'static str = "lang";
                const EXPR: &'static str = r#"|| -> Result<(), ()> { Err(())? }"#;
                Ok(if self.autocfg.probe_expression(EXPR) { Some(CATEGORY) } else { None })
            },
            "rust1" => {
                const CATEGORY: &'static str = "lang";
                Ok(Some(CATEGORY))
            },
            "step_trait" => {
                const CATEGORY: &'static str = "lib";
                const PATH: &'static str = "std::iter::Step";
                Ok(if self.autocfg.probe_path(PATH) { Some(CATEGORY) } else { None })
            },
            "unstable_features" => {
                const CATEGORY: &'static str = "comp";
                Ok(if self.version_check.channel.supports_features() { Some(CATEGORY) } else { None })
            },
            "unwrap_infallible" => {
                const CATEGORY: &'static str = "lib";
                const EXPR: &'static str = r#"Ok::<(), core::convert::Infallible>(()).into_ok()"#;
                Ok(if self.autocfg.probe_expression(EXPR) { Some(CATEGORY) } else { None })
            },
            _ => Err(UnsupportedFeatureTodoError::new(feature_name)),
        }
    }
}


#[cfg(test)]
mod tests
{
    extern crate create_temp_subdir;
    use super::{autocfg, CfgRustFeatures, ResultDynErr};

    impl CfgRustFeatures
    {
        fn for_test(name: &str) -> ResultDynErr<Self>
        {
            let out_dir = create_temp_subdir::TempSubDir::new(name).unwrap();
            let ac = try!(autocfg::AutoCfg::with_dir(&out_dir));
            Self::with_autocfg(ac)
        }
    }

    #[test]
    fn new()
    {
        assert!(CfgRustFeatures::for_test("unittest-lib-new").is_ok());
    }
}
