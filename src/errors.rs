use std::{
    error::Error,
    fmt,
};

/// Error that occurs when a feature name is unsupported by this crate currently.
#[derive(Debug)]
pub struct UnsupportedFeatureTodoError(String);

impl UnsupportedFeatureTodoError
{
    pub(crate) fn new(feature_name: &str) -> Self
    {
        Self(format!(
            "To request support for feature {:?}, open an issue at: {}",
            feature_name,
            env!("CARGO_PKG_REPOSITORY")
        ))
    }
}

impl Error for UnsupportedFeatureTodoError {}

impl fmt::Display for UnsupportedFeatureTodoError
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        f.write_str(&self.0)
    }
}


/// Error that occurs when [`version_check`] fails.
///
/// `version_check` does not provide its own error type, so we provide this.
#[derive(Debug)]
pub(crate) struct VersionCheckError;

impl Error for VersionCheckError {}

impl fmt::Display for VersionCheckError
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        f.write_str("version_check error")
    }
}
