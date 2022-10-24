// If provided by either stable or unstable feature, have this target be non-empty.
#![cfg(any(
    // Only set/true when the currently-used version of the cfg_rust_features crate supports it
    // and it is stable in the currently-used version of Rust.
    rust_lib_feature = "test",
    // Only set/true when a nightly (or dev) compiler is being used.
    rust_comp_feature = "unstable_features"
))]
// Else, a stable compiler version without the feature is being used, so have this target be
// empty to cause all the below items to be ignored as if they do not exist.
#![cfg_attr(
    // If the feature is still unstable
    not(rust_lib_feature = "test"),
    // then it needs to be specially enabled.
    feature(test)
)]
// Else if the feature is stable, #![feature(test)] is not needed.

// Valid whenever the feature is enabled, whether stable or unstable.
extern crate test;

#[bench]
fn dummy(bencher: &mut test::Bencher)
{
    bencher.iter(|| 1 + 1)
}
