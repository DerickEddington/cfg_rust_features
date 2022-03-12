//! The definition of which features are recognized by this crate.

use super::FeatureCategory;


/// Descriptor of a recognized feature.
///
/// (Actually private to the crate, not part of public API.  Is only `pub` for old Rust versions.)
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Feature
{
    pub name:       &'static str,
    pub categories: &'static [FeatureCategory],
    pub probe:      Probe,
}

/// How to test whether a `rustc` version provides a feature.
///
/// (Actually private to the crate, not part of public API.  Is only `pub` for old Rust versions.)
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Probe
{
    Expr(&'static str),
    Type(&'static str),
    Path(&'static str),
    AlwaysEnabled,
    UnstableFeatures,
}

/// The definition of which features are recognized by this crate.
///
/// Invariant: Must always be sorted by name.  Keep this in mind when making changes to it.  There
/// is a unit-test that checks this.
const DEFINITION: &'static [Feature] = &[
    Feature {
        name:       "cfg_version",
        categories: &["lang"],
        probe:      Probe::Expr(r#"{ #[cfg(version("1.0"))] struct X; X }"#),
    },
    Feature {
        name:       "destructuring_assignment",
        categories: &["lang"],
        probe:      Probe::Expr("{ let (_a, _b); (_a, _b) = (1, 2); }"),
    },
    Feature {
        name:       "inner_deref",
        categories: &["lib"],
        probe:      Probe::Expr("Ok::<_, ()>(vec![1]).as_deref()"),
    },
    Feature {
        name:       "iter_zip",
        categories: &["lib"],
        probe:      Probe::Path("std::iter::zip"),
    },
    Feature { name: "never_type", categories: &["lang"], probe: Probe::Type("!") },
    Feature {
        name:       "question_mark",
        categories: &["lang"],
        probe:      Probe::Expr("|| -> Result<(), ()> { Err(())? }"),
    },
    Feature {
        name:       "rust1",
        categories: &["comp", "lang", "lib"],
        probe:      Probe::AlwaysEnabled,
    },
    Feature {
        name:       "step_trait",
        categories: &["lib"],
        probe:      Probe::Path("std::iter::Step"),
    },
    Feature {
        name:       "unstable_features",
        categories: &["comp"],
        probe:      Probe::UnstableFeatures,
    },
    Feature {
        name:       "unwrap_infallible",
        categories: &["lib"],
        probe:      Probe::Expr("Ok::<(), core::convert::Infallible>(()).into_ok()"),
    },
];

/// Lookup a feature descriptor by name.  Return `None` if not recognized.
///
/// (Actually private to the crate, not part of public API.  Is only `pub` for old Rust versions.)
pub fn get(feature_name: &str) -> Option<&'static Feature>
{
    DEFINITION
        .binary_search_by(|element| element.name.cmp(feature_name))
        .ok()
        .map(|index| &DEFINITION[index])
}


#[cfg(test)]
mod tests
{
    use super::{Feature, DEFINITION};

    fn sorted() -> Vec<Feature>
    {
        let mut v = Vec::from(DEFINITION);
        v.sort_by(|a, b| a.name.cmp(b.name));
        v
    }

    #[test]
    fn no_duplicates()
    {
        let mut deduped = sorted();
        deduped.dedup();
        assert_eq!(DEFINITION, &*deduped);
    }

    #[test]
    fn is_sorted()
    {
        assert_eq!(DEFINITION, &*sorted());
    }
}
