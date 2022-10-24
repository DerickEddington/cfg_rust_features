/*!
A build-script helper to set `cfg` options according to probing which features of your choice are
enabled in the Rust compiler, language, and library, without reference to versions of Rust.

The primary purpose is to detect when previously-unstable features become stabilized, based on
feature presence and not on Rust version.  This helps design conditionally-compiled code that can
adjust whenever a feature becomes stable in whichever unknown future version of Rust.

The `cfg` options that are set are key-value forms like:
`rust_lib_feature = "iter_zip"`,
`rust_lang_feature = "never_type"`,
etc.

The probing does not use `#![feature(...)]` and so the options that are set represent features
that are stable, consistently with either `nightly` or `stable` compilers.  It is still possible
to conditionally enable unstable features, with the `rust_comp_feature = "unstable_features"`
option that can be detected and set when a `nightly` (or `dev`) compiler is used.

# Notes

- You must be careful about designing code around unstable features that could change before they
  are stabilized.

- Currently, this crate only supports a small subset of features (of both unstable and stable).
  You may request support for additional features, by opening an issue at:
  <https://github.com/DerickEddington/cfg_rust_features/issues>.

# Examples

- Your build script, usually `build.rs`, can be as simple as:
  ```no_run
  fn main() {
      let of_interest = ["iter_zip", /* Or: "unstable_features", etc ... */];
      cfg_rust_features::emit!(of_interest).unwrap();
  }
  ```

- To work with stable Rust versions, you implemented a workaround for the absence of an unstable
  feature that you wish you could use, and you do not know in which future version it will become
  stabilized (if ever), but you are confident that the API of this feature will not change before
  stabilizing.  So, with the help of this crate, you design conditional compilation that, if the
  feature becomes stable, marks your workaround as deprecated and uses the feature instead.

  If your workaround was to have an `into_ok` method on `Result<T, Infallible>`, such detection
  could be done like:
  ```rust
  # use std::convert::Infallible;
  #
  #[cfg_attr(rust_lib_feature = "unwrap_infallible", deprecated)]
  trait IntoOk { /* ... */ }

  #[cfg(not(rust_lib_feature = "unwrap_infallible"))]
  impl<T> IntoOk for Result<T, Infallible> { /* ... */ }
  ```

- To enable unstable features only when using a `nightly` (or `dev`) compiler:
  ```rust
  #![cfg_attr(rust_comp_feature = "unstable_features", feature(step_trait))]

  #[cfg(rust_comp_feature = "unstable_features")]
  fn maybe_use_step_trait() { /* ... */ }
  ```
  This avoids needing some Cargo package feature (e.g. `"unstable"`) for this, which some projects
  might prefer.

  Or, enabling unstable features can be done only when the feature is not yet stabilized, and not
  done if/when a feature becomes stable, like:
  ```ignore
  #![cfg_attr(not(rust_lib_feature = "step_trait"), feature(step_trait))]

  fn assume_step_trait_is_available() { /* ... */ }
  ```

  Or, a package could anticipate that future versions of itself will have breaking changes due to
  plans to adopt some Rust features if/when they become stable, and the package could provide a
  non-default Cargo package feature that enables building like this in order to experiment with
  this, while the default and other package features continue to uphold the stability of the API.
  This anticipatory package feature can be made to automatically use either the stable or unstable
  Rust feature, so that it works both before and after a Rust feature is stabilized, before
  developing changes to the stable API, by doing something like:
  ```rust
  #![cfg_attr(
      all(feature = "anticipate", not(rust_lib_feature = "step_trait")),
      feature(step_trait)
  )]

  # macro_rules! cfg_if { ( $( $x:tt )* ) => {} }
  #
  cfg_if! {
      if #[cfg(feature = "anticipate")] {
          // Break the API to use anticipated Rust features,
          // whether still unstable or recently stable.
          pub fn assume_step_trait_is_available() { /* ... */ }
      }
      else {
          // Stable API that works with older stable versions of Rust.
          pub fn do_not_use_step_trait() { /* ... */ }
      }
  }
  ```
  (Note: This would not follow the recommended convention that package [features should be
  additive](https://doc.rust-lang.org/1.64.0/cargo/reference/features.html#semver-compatibility),
  but some projects might be ok with this, because the purpose of such an `"anticipate"` feature
  is very limited and clear and so users of it should know to not use it for their stable needs,
  and because this approach can help avoid needing a separate branch to have such experimental
  changes and this could help keep development of both the stable and experimental APIs in-sync.)

- To have benchmarks (which (as of 2022-10-23) require a `nightly` compiler) that do not interfere
  with using a `stable` compiler, without needing some extra package feature.  This enables using
  Cargo options like `--all-targets` (which includes `--benches`) with a `stable` compiler without
  error, which can be especially helpful with tools which use that.  This is done, at the top of
  some `benches/whatever.rs`, like:
  ```ignore
  #![cfg(rust_comp_feature = "unstable_features")]
  /* ... */
  ```
  and thus `benches/` targets are effectively empty with a `stable` compiler but are non-empty
  with `nightly`, automatically without needing to remember to give `--features`.

  Further, targets can be made to adjust if a future version of Rust stabilizes a feature,
  e.g. the benchmarking `test` feature, and if a future version of this crate adds support for
  that feature; and targets can still be made to work while the feature is unstable and while this
  crate does not have support, like:
  ```ignore
  // If provided by either stable or unstable feature, have this target
  // be non-empty.
  #![cfg(any(
      // Only set/true when the currently-used version of the
      // cfg_rust_features crate supports it and it is stable in the
      // currently-used version of Rust.
      rust_lib_feature = "test",
      // Only set/true when a nightly (or dev) compiler is being used.
      rust_comp_feature = "unstable_features"
  ))]
  // Else, a stable compiler version without the feature is being used,
  // so have this target be empty to cause all the below items to be
  // ignored as if they do not exist.

  #![cfg_attr(
      // If the feature is still unstable
      not(rust_lib_feature = "test"),
      // then it needs to be specially enabled.
      feature(test)
  )]
  // Else if the feature is stable, #![feature(test)] is not needed.

  // Valid whenever the feature is enabled, whether stable or unstable.
  extern crate test;

  /* ... */
  ```
  and thus this code, at the top of the file at least, should not need to be changed both when the
  feature is unstable and when it later becomes stable (unless the feature itself changes while
  unstable, of course); and, also, this code will continue to be valid with older versions of Rust
  where the feature is considered unstable even after a newer version stabilizes it.

# Stability Policy

The API follows the normal Cargo SemVer policy, with the qualification that it is allowed for the
error behavior of future versions having the same primary number to change somewhat:

- Future versions may change to support additional feature names and so no longer error for those.
  But once a feature name is supported it will not be removed and so will never error for that and
  future versions.

- Future versions may change to possibly return different `Error` types behind `dyn Error` when
  creating instances of `CfgRustFeatures`, due to internal changes in how the probing is done and
  in which dependencies are used.  But the use of the `Box<dyn Error>` type will remain stable.

# Minimum Supported Rust Version

Rust `1.0.0` will always be supported, so this crate can be used by other crates which support
that old version.

# Documentation

The source-code has doc comments, which are rendered as the API documentation.

View online at: <http://docs.rs/cfg_rust_features>

Or, you can generate them yourself and view locally by doing:

```shell
cargo doc --open
```
 */
// Remember to run `cargo readme --no-license > README.md` and re-adjust the fenced code blocks to
// be of type `rust`, when changing the above doc-comment.

// These lints are chosen to work with Rust versions as old as 1.0.0.
#![forbid(unsafe_code)]
#![allow(unknown_lints)]
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
// (Must be after the above `warn` groups, for (some of) these to have effect.)
#![allow(deprecated, bare_trait_objects, absolute_paths_not_starting_with_crate, keyword_idents)]
// Warn about this one but avoid annoying hits for dev-dependencies.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]


extern crate autocfg;
extern crate version_check;

mod errors;
mod helpers;
mod recognized;

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::hash::Hash;
use std::iter::FromIterator;

pub use errors::UnsupportedFeatureTodoError;
use errors::{unsupported_feature_todo_error, VersionCheckError};
pub use helpers::emit_warning;
use recognized::Probe;


/// Name of a feature, as recognized by this crate.
pub trait FeatureName: Borrow<str> + Eq + Hash {}
impl<T: Borrow<str> + Eq + Hash> FeatureName for T {}

/// Name of a feature category, as defined by this crate.
pub type FeatureCategory = &'static str;
/// Set of feature categories that a feature belongs to.
pub type FeatureCategories = HashSet<FeatureCategory>;
/// Whether a feature is enabled and its categories if so.
pub type FeatureEnabled = Option<FeatureCategories>;
/// Indicates whether each from a set of features was found to be enabled and its categories.
pub type EnabledFeatures<F> = HashMap<F, FeatureEnabled>;

/// Rust 1.0.0 does not support the `dyn` keyword.  This helps be clearer.
pub type ResultDynErr<T> = Result<T, Box<Error>>;


/// Helper that does the common basic use of this crate.  Suitable as the body of the `main`
/// function of a build script.
///
/// Calls [`CfgRustFeatures::emit_multiple`] on a temporary instance with the given features'
/// names.  Also calls [`emit_rerun_if_changed_file`] with the name of the file in which this
/// macro was invoked.
///
/// # Examples
/// A `build.rs` can be as simple as:
/// ```no_run
/// fn main() {
///     cfg_rust_features::emit!(["iter_zip"]).unwrap();
/// }
/// ```
#[macro_export]
macro_rules! emit {
    ($features_names:expr) => {{
        $crate::emit_rerun_if_changed_file(file!());
        $crate::CfgRustFeatures::emit($features_names).map(|_| ())
    }};
}


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
    /// Convenience that calls [`Self::emit_multiple`] on a temporary instance.
    pub fn emit<F: FeatureName, I: IntoIterator<Item = F>>(
        features_names: I
    ) -> ResultDynErr<EnabledFeatures<F>>
    {
        Ok(try!(try!(CfgRustFeatures::new()).emit_multiple(features_names)))
    }

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
                autocfg:       autocfg,
                version_check: VersionCheck { version: version, channel: channel, date: date },
            })
        }
        else {
            Err(VersionCheckError.into())
        }
    }

    /// Write, to `stdout`, instructions for Cargo to set configuration options that indicate
    /// whether the currently-used version of Rust (compiler, language, and library) has enabled
    /// the given sequence of features.
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
    /// #     let dir = TempSubDir::new("doctest-emit_multiple").unwrap();
    /// #     std::env::set_var("OUT_DIR", &dir);
    /// #
    /// #     fn make_try_work() -> ResultDynErr<()> {
    /// let gathered_info_instance = try!(CfgRustFeatures::new());
    /// let enabled_features = try!(gathered_info_instance.emit_multiple(vec![
    ///     "arbitrary_self_types",
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
    /// and its categories if so.  May be ignored, since the instructions for Cargo are also
    /// written out.
    ///
    /// # Errors
    ///
    /// If a feature name is unsupported by the current version of this crate.  The message will
    /// show the URL where a new issue may be opened to request adding support for the feature.
    ///
    /// Note: This crate's stability policy allows for this error behavior to change somewhat:
    /// future versions having the same primary number may change to support additional feature
    /// names and so no longer error for those; but once a feature name is supported it will not
    /// be removed and so will never error for that and future versions.
    pub fn emit_multiple<F: FeatureName, I: IntoIterator<Item = F>>(
        &self,
        features_names: I,
    ) -> Result<EnabledFeatures<F>, UnsupportedFeatureTodoError>
    {
        let enabled_features = try!(self.probe_multiple(features_names));

        for (name, enabled) in &enabled_features {
            self.emit_single(name.borrow(), enabled);
        }
        Ok(enabled_features)
    }

    /// Like [`Self::emit_multiple`] but does not write anything.  Use when only the return value
    /// is of interest.
    ///
    /// # Returns
    /// Same as [`Self::emit_multiple`].
    ///
    /// # Errors
    /// Same as [`Self::emit_multiple`].
    pub fn probe_multiple<F: FeatureName, I: IntoIterator<Item = F>>(
        &self,
        features_names: I,
    ) -> Result<EnabledFeatures<F>, UnsupportedFeatureTodoError>
    {
        let mut enabled_features = HashMap::new();

        for name in features_names {
            let enabled = try!(self.probe_single(name.borrow()));
            let _ = enabled_features.insert(name, enabled);
        }
        Ok(enabled_features)
    }

    fn emit_single(
        &self,
        feature_name: &str,
        enabled: &FeatureEnabled,
    )
    {
        if let &Some(ref categories) = enabled {
            for category in categories {
                helpers::emit_rust_feature(category, feature_name);
            }
        }
    }

    /// Tests whether the current `rustc` provides the given compiler/language/library feature as
    /// stable (i.e. without needing the `#![feature(...)]` of nightly).
    ///
    /// # Returns
    /// The categories of the feature if the feature is enabled, or else `None`.
    ///
    /// # Errors
    /// If the feature name is unsupported by this crate currently.
    fn probe_single(
        &self,
        feature_name: &str,
    ) -> Result<FeatureEnabled, UnsupportedFeatureTodoError>
    {
        let feature = try!(
            recognized::get(feature_name)
                .ok_or_else(|| unsupported_feature_todo_error(feature_name))
        );
        let enabled = match feature.probe {
            Probe::Expr(e) => self.autocfg.probe_expression(e),
            Probe::Type(t) => self.autocfg.probe_type(t),
            Probe::Path(p) => self.autocfg.probe_path(p),
            Probe::AlwaysEnabled => true,
            Probe::UnstableFeatures => self.version_check.channel.supports_features(),
        };
        Ok(if enabled {
            Some(HashSet::from_iter(feature.categories.iter().map(|&x| x)))
        }
        else {
            None
        })
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

    #[test]
    fn error()
    {
        use std::error::Error;

        let features_names = vec!["rust1", "bogusness", "dummy"];
        let cfg_rust_features = CfgRustFeatures::for_test("unittest-lib-error").unwrap();
        let result = cfg_rust_features.emit_multiple(features_names);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().description(),
                   "To request support for feature \"bogusness\", open an issue at: \
                    https://github.com/DerickEddington/cfg_rust_features");
    }

    #[test]
    fn generic()
    {
        use std::borrow::Cow;
        use std::collections::BTreeSet;
        use std::iter::FromIterator;

        let cfg_rust_features = CfgRustFeatures::for_test("unittest-lib-generic").unwrap();
        {
            let features_names = vec![String::from("rust1")];
            let _enabled_features = cfg_rust_features.emit_multiple(features_names).unwrap();
        }
        {
            let features_names = BTreeSet::from_iter(vec![Cow::from("rust1")]);
            let _enabled_features = cfg_rust_features.emit_multiple(features_names).unwrap();
        }
    }
}
