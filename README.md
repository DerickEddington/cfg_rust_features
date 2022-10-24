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

- To have benchmarks (which require a `nightly` compiler) that do not interfere with using a
  `stable` compiler, without needing some extra package feature.  This enables using Cargo options
  like `--all-targets` (which includes `--benches`) with a `stable` compiler without error, which
  can be especially helpful with IDE tools which use that.  This is done, at the top of some
  `benches/whatever.rs`, like:
  ```rust
  #![cfg(rust_comp_feature = "unstable_features")]
  /* ... */
  ```
  and thus `benches/` targets are effectively empty with a `stable` compiler but are non-empty
  with `nightly`, automatically without needing to remember to give `--features`.

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
