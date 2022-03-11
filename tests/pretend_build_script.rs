// Note: This will print to stderr what look like errors but these are only from autocfg doing the
// intended probing (as it runs its own rustc commands that expectedly might have compiler
// errors), and this will also print the build-script instructions to stdout, and these prints
// will be intermixed (and their order is randomized, due to the current internal iteration of a
// HashMap).  It can be helpful to redirect these, e.g.:
//   cargo test --test pretend_build_script 2> /dev/null

#![allow(unknown_lints, deprecated, bare_trait_objects)]

extern crate cfg_rust_features;
extern crate create_temp_subdir;

use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::hash::Hash;
use std::iter::FromIterator;

use cfg_rust_features::{
    emit_rerun_if_changed_file, CfgRustFeatures, EnabledFeatures, FeatureCategory, FeatureName,
};
use create_temp_subdir::TempSubDir;

type ResultDynErr<T> = Result<T, Box<Error>>;


/// Like a `main` function of a build script (modulo the `Ok` type).
fn pretend_build_script() -> ResultDynErr<EnabledFeatures<'static>>
{
    emit_rerun_if_changed_file(file!());

    Ok(try!(try!(CfgRustFeatures::new()).emit_rust_features(vec![
        // "cfg_version",  // Omitted to exercise not giving a supported one.
        "inner_deref",
        "destructuring_assignment",
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

    assert_results(&pretend_build_script().unwrap());
}


/// Must correspond to what [`pretend_build_script`] emits.
fn assert_results(call_result: &EnabledFeatures<'static>)
{
    fn assert_enabled_fits_required_and_allowed<T: Hash + Eq>(
        enabled: HashSet<T>,
        required: HashSet<T>,
        allowed: HashSet<T>,
    )
    {
        assert!(enabled.is_superset(&required));
        assert!(enabled.is_subset(&allowed));
    }

    #[derive(Hash, Eq, PartialEq, Clone, Copy)]
    struct Feature
    {
        category: FeatureCategory,
        name:     FeatureName<'static>,
    }

    let required_features =
        HashSet::from_iter(vec![Feature { category: "lang", name: "rust1" }]);
    let optional_features = HashSet::from_iter(vec![
        Feature { category: "comp", name: "unstable_features" },
        Feature { category: "lang", name: "destructuring_assignment" },
        Feature { category: "lang", name: "never_type" },
        Feature { category: "lang", name: "question_mark" },
        Feature { category: "lib", name: "inner_deref" },
        Feature { category: "lib", name: "iter_zip" },
        Feature { category: "lib", name: "step_trait" },
        Feature { category: "lib", name: "unwrap_infallible" },
    ]);
    let allowed_features = &required_features | &optional_features;

    // Check the EnabledFeatures HashMap value, returned by the call to
    // CfgRustFeatures::emit_rust_features, which indicates whether each of the chosen features
    // was found to be enabled and its category if so.
    {
        type Enabled = HashSet<(FeatureName<'static>, FeatureCategory)>;

        fn from_hashmap(hashmap: &EnabledFeatures<'static>) -> Enabled
        {
            hashmap.iter().filter_map(|(&k, v)| v.map(|c| (k, c))).collect()
        }

        fn from_hashset(hashset: &HashSet<Feature>) -> Enabled
        {
            hashset.iter().map(|feat| (feat.name, feat.category)).collect()
        }

        let enabled = from_hashmap(call_result);
        let required = from_hashset(&required_features);
        let allowed = from_hashset(&allowed_features);

        assert_enabled_fits_required_and_allowed(enabled, required, allowed);
    }
}
