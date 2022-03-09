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


mod error;
mod helpers;

pub use error::Error;
use {
    error::VersionCheckError,
    helpers::{
        emit_cargo_instruction,
        emit_rust_feature,
        emit_warning,
    },
    std::collections::HashMap,
};


const REPO_ISSUES_URL: &str = "https://github.com/DerickEddington/cfg_rust_features/issues";


/// Tell Cargo to not default to scanning the entire package directory for changes, but to check
/// only given files, when deciding if a build script needs to be rerun.
///
/// Intended to be used once per each of the file(s) of a build script.
pub fn emit_rerun_if_changed_file(filename: &str)
{
    emit_cargo_instruction("rerun-if-changed", Some(filename));
}


/// Information about the current Rust compiler.
///
/// Gathered when a [new intance is created](CfgRustFeatures::new).  Used to emit
/// [conditional-compilation configuration-options for use with the `cfg` et al
/// attributes](https://doc.rust-lang.org/reference/conditional-compilation.html).
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
    /// Gather the information about the current Rust compiler, and return the value that can
    /// perform the operations with it.
    ///
    /// # Errors
    /// If the information gathering fails.  (E.g. if `rustc` could not be run.)
    pub fn new() -> Result<Self, Error>
    {
        Ok(Self {
            autocfg:       autocfg::AutoCfg::new()?,
            version_check: {
                let (version, channel, date) =
                    version_check::triple().ok_or(VersionCheckError)?;
                VersionCheck { version, channel, date }
            },
        })
    }

    /// Set configuration options that indicate whether the currently-used version of Rust
    /// (compiler, language, and library) supports the given sequence of feature names.
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
    /// # use cfg_rust_features::{CfgRustFeatures, Error};
    /// # fn main() -> Result<(), Error> {
    /// CfgRustFeatures::new()?.emit_rust_features([
    ///     "never_type",
    ///     "step_trait",
    ///     "unwrap_infallible",
    ///     "unstable_features",
    /// ]);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// will set the following configuration options:
    ///
    /// ```text
    /// TODO
    /// ```
    ///
    /// # Returns
    ///
    /// A [`HashMap`] that indicates whether each of the given features was found to be supported
    /// or not.
    ///
    /// # Panics
    ///
    /// If a feature name is unsupported currently.  The panic message will show the URL where a
    /// new issue may be opened to request adding support for the feature.
    pub fn emit_rust_features<'l>(
        &self,
        features: impl IntoIterator<Item = &'l str>,
    ) -> HashMap<&'l str, bool>
    {
        use core::iter::repeat;

        let mut features: HashMap<_, _> = features.into_iter().zip(repeat(false)).collect();
        let mut any_stable_rust_feature = false;

        for (feature, is_stable) in &mut features {
            *is_stable = self.emit_rust_feature(feature);
            any_stable_rust_feature = *is_stable || any_stable_rust_feature;
        }
        if any_stable_rust_feature && self.probe_rust_feature("cfg_version").is_some() {
            emit_warning("Rust feature cfg_version is now stable. Consider using instead.");
        }
        features
    }

    fn emit_rust_feature(
        &self,
        feature: &str,
    ) -> bool
    {
        if let Some(key_category) = self.probe_rust_feature(feature) {
            emit_rust_feature(key_category, feature);
            true
        }
        else {
            false
        }
    }

    /// Tests whether the current `rustc` provides the given compiler/language/library feature as
    /// stable (i.e. without needing the `#![feature(...)]` of nightly).
    ///
    /// `feature`: One of the "feature flags" named by
    /// <https://doc.rust-lang.org/nightly/unstable-book/index.html>.
    fn probe_rust_feature(
        &self,
        feature: &str,
    ) -> Option<&'static str>
    {
        // TODO: Could improve with some static CATEGORY_TABLE: Once that associates feature to
        // category, which would allow factoring-out redundant `const CATEGORY` and redundant
        // `.then(|| ...)`.

        match feature {
            "cfg_version" => {
                const CATEGORY: &str = "lang";
                const EXPR: &str = r#"{ #[cfg(version("1.0"))] struct X; X }"#;
                self.autocfg.probe_expression(EXPR).then(|| CATEGORY)
            },
            "step_trait" => {
                const CATEGORY: &str = "lib";
                const PATH: &str = "std::iter::Step";
                self.autocfg.probe_path(PATH).then(|| CATEGORY)
            },
            "unstable_features" => {
                const CATEGORY: &str = "comp";
                self.version_check.channel.supports_features().then(|| CATEGORY)
            },
            "unwrap_infallible" => {
                const CATEGORY: &str = "lib";
                const EXPR: &str = r#"Ok::<(), core::convert::Infallible>(()).into_ok()"#;
                self.autocfg.probe_expression(EXPR).then(|| CATEGORY)
            },
            #[allow(clippy::todo)]
            _ => todo!("Request support for feature {:?} at: {}", feature, REPO_ISSUES_URL),
        }
    }
}

