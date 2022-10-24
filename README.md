# cfg_rust_features

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

## Notes

- You must be careful about designing code around unstable features that could change before they
  are stabilized.

- Currently, this crate only supports a small subset of features (of both unstable and stable).
  You may request support for additional features, by opening an issue at:
  <https://github.com/DerickEddington/cfg_rust_features/issues>.

## Examples

- Your build script, usually `build.rs`, can be as simple as:
  ```rust
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
  ```rust
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
  ```rust
  #![cfg(rust_comp_feature = "unstable_features")]
  /* ... */
  ```
  and thus `benches/` targets are effectively empty with a `stable` compiler but are non-empty
  with `nightly`, automatically without needing to remember to give `--features`.

  Further, targets can be made to adjust if a future version of Rust stabilizes a feature,
  e.g. the benchmarking `test` feature, and if a future version of this crate adds support for
  that feature; and targets can still be made to work while the feature is unstable and while this
  crate does not have support, like:
  ```rust
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

## Stability Policy

The API follows the normal Cargo SemVer policy, with the qualification that it is allowed for the
error behavior of future versions having the same primary number to change somewhat:

- Future versions may change to support additional feature names and so no longer error for those.
  But once a feature name is supported it will not be removed and so will never error for that and
  future versions.

- Future versions may change to possibly return different `Error` types behind `dyn Error` when
  creating instances of `CfgRustFeatures`, due to internal changes in how the probing is done and
  in which dependencies are used.  But the use of the `Box<dyn Error>` type will remain stable.

## Minimum Supported Rust Version

Rust `1.0.0` will always be supported, so this crate can be used by other crates which support
that old version.

## Documentation

The source-code has doc comments, which are rendered as the API documentation.

View online at: <http://docs.rs/cfg_rust_features>

Or, you can generate them yourself and view locally by doing:

```shell
cargo doc --open
```
