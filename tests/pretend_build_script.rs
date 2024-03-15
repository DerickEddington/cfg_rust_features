// Note: This will print to stderr what look like errors but these are only from autocfg doing the
// intended probing (as it runs its own rustc commands that expectedly might have compiler
// errors), and this will also print the build-script instructions to stdout, and these prints
// will be intermixed (and their order is randomized, due to the current internal iteration of a
// HashMap).  It can be helpful to redirect these, e.g.:
//   cargo test --test pretend_build_script 2> /dev/null

#![allow(unknown_lints, deprecated, bare_trait_objects)]

extern crate cfg_rust_features;
extern crate create_temp_subdir;

use std::collections::{BTreeSet, HashSet};
use std::env;
use std::error::Error;
use std::hash::Hash;
use std::iter::FromIterator;

use cfg_rust_features::{emit_rerun_if_changed_file, CfgRustFeatures, FeatureCategory};
use create_temp_subdir::TempSubDir;

type ResultDynErr<T> = Result<T, Box<Error>>;

type FeatureName = &'static str;
type EnabledFeatures = cfg_rust_features::EnabledFeatures<FeatureName>;


/// Like a `main` function of a build script (modulo the `Ok` type).
fn pretend_build_script() -> ResultDynErr<EnabledFeatures>
{
    emit_rerun_if_changed_file(file!());

    Ok(try!(try!(CfgRustFeatures::new()).emit_multiple(vec![
        "arbitrary_self_types",
        // "cfg_version",  // Omitted to exercise not giving a supported one.
        "inner_deref",
        "destructuring_assignment",
        "error_in_core",
        "iter_zip",
        "never_type",
        "question_mark",
        "rust1",
        "step_trait",
        "unstable_features",
        "unwrap_infallible",
    ])))
}


fn main()
{
    // Setup to pretend that this program is a build script.
    let out_dir = TempSubDir::new("intgtest-pretend_build_script").unwrap();
    env::set_var("OUT_DIR", &out_dir);

    assert_enabled_features(&pretend_build_script().unwrap());
}


/// Check the `EnabledFeatures` `HashMap` value, returned by the call to
/// `CfgRustFeatures::emit_multiple`, which indicates whether each of the chosen features was
/// found to be enabled and its categories if so.
///
/// Must correspond to what [`pretend_build_script`] emits.
fn assert_enabled_features(enabled: &EnabledFeatures)
{
    /// Element of `HashSet`s.  Similar shape as a Set iterator yields.  `BTreeSet` needed because
    /// it `impl`s `Hash`.
    type Feature = (FeatureName, BTreeSet<FeatureCategory>);

    macro_rules! set {
        [$t:ty: $($e:expr),*] => {
            <$t>::from_iter(vec![$($e),*])
        }
    }
    macro_rules! hset {
        [$($rest:tt)*] => {
            set![HashSet<_>: $($rest)*]
        }
    }
    macro_rules! bset {
        [$($rest:tt)*] => {
            set![BTreeSet<_>: $($rest)*]
        }
    }

    fn bset_from_hset<T: Clone + Hash + Ord>(hset: &HashSet<T>) -> BTreeSet<T>
    {
        hset.iter().cloned().collect()
    }

    fn from_enabled_features(enabled_features: &EnabledFeatures) -> HashSet<Feature>
    {
        enabled_features
            .iter()
            .filter_map(|(&k, v)| v.as_ref().map(|c| (k, bset_from_hset(c))))
            .collect()
    }

    fn assert_enabled_fits_required_and_allowed<T: Hash + Eq>(
        enabled: &HashSet<T>,
        required: &HashSet<T>,
        allowed: &HashSet<T>,
    )
    {
        assert!(enabled.is_superset(required));
        assert!(enabled.is_subset(allowed));
    }


    let required = hset![("rust1", bset!["comp", "lang", "lib"])];
    let optional = hset![
        ("unstable_features", bset!["comp"]),
        ("arbitrary_self_types", bset!["lang"]),
        ("destructuring_assignment", bset!["lang"]),
        ("never_type", bset!["lang"]),
        ("question_mark", bset!["lang"]),
        ("error_in_core", bset!["lib"]),
        ("inner_deref", bset!["lib"]),
        ("iter_zip", bset!["lib"]),
        ("step_trait", bset!["lib"]),
        ("unwrap_infallible", bset!["lib"])
    ];
    let allowed = &required | &optional;

    let enabled = from_enabled_features(enabled);
    assert_enabled_fits_required_and_allowed(&enabled, &required, &allowed);
}
