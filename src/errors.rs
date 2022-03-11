use std::error::Error;
use std::fmt;


/// Error that occurs when a feature name is unsupported by this crate currently.
#[derive(Debug)]
pub struct UnsupportedFeatureTodoError(String);

impl UnsupportedFeatureTodoError
{
    /// (Actually private to the crate, not part of public API.  Is only `pub` for old Rust
    /// versions.)
    pub fn new(feature_name: &str) -> Self
    {
        UnsupportedFeatureTodoError(format!(
            "To request support for feature {:?}, open an issue at: {}",
            feature_name, "https://github.com/DerickEddington/cfg_rust_features"
        ))
    }
}

impl Error for UnsupportedFeatureTodoError
{
    fn description(&self) -> &str
    {
        &self.0
    }
}

impl fmt::Display for UnsupportedFeatureTodoError
{
    fn fmt<'f>(
        &self,
        f: &mut fmt::Formatter<'f>,
    ) -> fmt::Result
    {
        f.write_str(&self.0)
    }
}


/// Error that occurs when [`version_check`] fails.
///
/// `version_check` does not provide its own error type, so we provide this.
///
/// (Actually private to the crate, not part of public API.  Is only `pub` for old Rust versions.)
#[derive(Debug)]
pub struct VersionCheckError;

impl Error for VersionCheckError
{
    fn description(&self) -> &str
    {
        "version_check error"
    }
}

impl fmt::Display for VersionCheckError
{
    fn fmt<'f>(
        &self,
        f: &mut fmt::Formatter<'f>,
    ) -> fmt::Result
    {
        f.write_str(self.description())
    }
}
