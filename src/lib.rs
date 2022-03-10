#![cfg_attr(unix, doc = include_str!("../README.md"))]
#![cfg_attr(windows, doc = include_str!("..\\README.md"))]
#![forbid(unsafe_code)]
// Warn about desired lints that would otherwise be allowed by default.
#![warn(
    // Groups
    future_incompatible,
    nonstandard_style,
    rust_2018_compatibility, // unsure if needed with edition="2018"
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::cargo,
    // Individual lints not included in above groups and desired.
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    // missing_doc_code_examples, // maybe someday
    noop_method_call,
    pointer_structural_match,
    single_use_lifetimes, // annoying hits on invisible derived impls
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences,
)]
// Warn about this one but avoid annoying hits for dev-dependencies.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
// Exclude (re-allow) undesired lints included in above groups.
#![allow(
    clippy::missing_inline_in_public_items,
    clippy::implicit_return,
    clippy::blanket_clippy_restriction_lints,
    clippy::default_numeric_fallback,
    clippy::separated_literal_suffix,
    clippy::missing_docs_in_private_items,
    clippy::pattern_type_mismatch,
    clippy::shadow_reuse
)]


mod errors;
mod helpers;


pub use {
    errors::UnsupportedFeatureTodoError,
    helpers::emit_warning,
};
use {
    errors::VersionCheckError,
    std::{
        collections::HashMap,
        error::Error,
    },
};


/// Name of a feature, as recognized by this crate.
pub type FeatureName<'l> = &'l str;
/// Name of a feature category, as recognized by this crate.
pub type FeatureCategory = &'static str;
/// Whether a feature is enabled and its category if so.
pub type FeatureEnabled = Option<FeatureCategory>;
/// Indicates whether a set of features was found to be enabled and the category of each.
pub type EnabledFeatures<'l> = HashMap<FeatureName<'l>, FeatureEnabled>;


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
/// Intended to be called from a package's build script.
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
    pub fn new() -> Result<Self, Box<dyn Error>>
    {
        Self::with_autocfg(autocfg::AutoCfg::new()?)
    }

    fn with_autocfg(autocfg: autocfg::AutoCfg) -> Result<Self, Box<dyn Error>>
    {
        if let Some((version, channel, date)) = version_check::triple() {
            Ok(Self { autocfg, version_check: VersionCheck { version, channel, date } })
        }
        else {
            Err(VersionCheckError.into())
        }
    }

    /// Set configuration options that indicate whether the currently-used version of Rust
    /// (compiler, language, and library) supports the given sequence of feature names.
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
    /// # use std::error::Error;
    /// # use cfg_rust_features::CfgRustFeatures;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// #     let dir = tempfile::tempdir().unwrap();
    /// #     std::env::set_var("OUT_DIR", dir.path());
    /// #
    /// CfgRustFeatures::new()?.emit_rust_features([
    ///     "cfg_version",
    ///     "destructuring_assignment",
    ///     "inner_deref",
    ///     "iter_zip",
    ///     "never_type",
    ///     "question_mark",
    ///     "step_trait",
    ///     "unwrap_infallible",
    ///     "unstable_features",
    /// ])?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// with `rustc` version `1.56`, will write to `stdout`:
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
    /// and its category if so.
    ///
    /// # Errors
    ///
    /// If a feature name is unsupported by this crate currently.  The message will show the URL
    /// where a new issue may be opened to request adding support for the feature.
    pub fn emit_rust_features<'l>(
        &self,
        features_names: impl IntoIterator<Item = FeatureName<'l>>,
    ) -> Result<EnabledFeatures<'l>, UnsupportedFeatureTodoError>
    {
        use core::iter::repeat;

        let mut features_enabled: HashMap<_, _> =
            features_names.into_iter().zip(repeat(None)).collect();
        let mut any_stable_rust_feature = false;

        for (feature_name, enabled) in &mut features_enabled {
            *enabled = self.emit_rust_feature(feature_name)?;
            any_stable_rust_feature = enabled.is_some() || any_stable_rust_feature;
        }
        if any_stable_rust_feature && self.probe_rust_feature("cfg_version")?.is_some() {
            emit_warning("Rust feature cfg_version is now stable. Consider using instead.");
        }
        Ok(features_enabled)
    }

    fn emit_rust_feature(
        &self,
        feature_name: FeatureName<'_>,
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
    fn probe_rust_feature(
        &self,
        feature_name: FeatureName<'_>,
    ) -> Result<FeatureEnabled, UnsupportedFeatureTodoError>
    {
        // TODO: Could improve with some static CATEGORY_TABLE: Once that associates feature to
        // category, which would allow factoring-out redundant `const CATEGORY` and redundant
        // `.then(|| ...)`.

        match feature_name {
            "cfg_version" => {
                const CATEGORY: &str = "lang";
                const EXPR: &str = r#"{ #[cfg(version("1.0"))] struct X; X }"#;
                Ok(self.autocfg.probe_expression(EXPR).then(|| CATEGORY))
            },
            "destructuring_assignment" => {
                const CATEGORY: &str = "lang";
                const EXPR: &str = r#"{ let (a, b); (a, b) = (1, 2); }"#;
                Ok(self.autocfg.probe_expression(EXPR).then(|| CATEGORY))
            },
            "inner_deref" => {
                const CATEGORY: &str = "lib";
                const EXPR: &str = r#"Ok::<_, ()>(vec![1]).as_deref()"#;
                Ok(self.autocfg.probe_expression(EXPR).then(|| CATEGORY))
            },
            "iter_zip" => {
                const CATEGORY: &str = "lib";
                const EXPR: &str = r#"std::iter::zip([1], ['a'])"#;
                Ok(self.autocfg.probe_expression(EXPR).then(|| CATEGORY))
            },
            "never_type" => {
                const CATEGORY: &str = "lang";
                const TYPE: &str = "!";
                Ok(self.autocfg.probe_type(TYPE).then(|| CATEGORY))
            },
            "question_mark" => {
                const CATEGORY: &str = "lang";
                const EXPR: &str = r#"|| -> Result<(), ()> { Err(())? }"#;
                Ok(self.autocfg.probe_expression(EXPR).then(|| CATEGORY))
            },
            "step_trait" => {
                const CATEGORY: &str = "lib";
                const PATH: &str = "std::iter::Step";
                Ok(self.autocfg.probe_path(PATH).then(|| CATEGORY))
            },
            "unstable_features" => {
                const CATEGORY: &str = "comp";
                Ok(self.version_check.channel.supports_features().then(|| CATEGORY))
            },
            "unwrap_infallible" => {
                const CATEGORY: &str = "lib";
                const EXPR: &str = r#"Ok::<(), core::convert::Infallible>(()).into_ok()"#;
                Ok(self.autocfg.probe_expression(EXPR).then(|| CATEGORY))
            },
            _ => Err(UnsupportedFeatureTodoError::new(feature_name)),
        }
    }
}


#[cfg(test)]
mod tests
{
    use {
        super::CfgRustFeatures,
        std::error::Error,
        tempfile::tempdir,
    };

    #[allow(clippy::multiple_inherent_impl)]
    impl CfgRustFeatures
    {
        fn for_test() -> Result<Self, Box<dyn Error>>
        {
            let out_dir = tempdir()?;
            let ac = autocfg::AutoCfg::with_dir(out_dir.path())?;
            let it = Self::with_autocfg(ac)?;
            out_dir.close()?;
            Ok(it)
        }
    }

    #[test]
    fn new() -> Result<(), Box<dyn Error>>
    {
        drop(CfgRustFeatures::for_test()?);
        Ok(())
    }
}
