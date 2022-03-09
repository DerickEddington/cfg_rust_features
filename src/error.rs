use std::fmt;

/// Error type that might occur when trying to gather information about the Rust compiler.
///
/// Opaque because this makes no guarantees about the internal details, but does implement
/// [`std::error::Error`] which may be used to get the source of an error.
#[derive(Debug)]
pub struct Error(Kind);

#[derive(Debug)]
enum Kind
{
    AutoCfg(autocfg::Error),
    VersionCheck,
}

impl From<autocfg::Error> for Error
{
    fn from(e: autocfg::Error) -> Self
    {
        Self(Kind::AutoCfg(e))
    }
}

pub(crate) struct VersionCheckError;

impl From<VersionCheckError> for Error
{
    fn from(_: VersionCheckError) -> Self
    {
        Self(Kind::VersionCheck)
    }
}

impl fmt::Display for Error
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        match &self.0 {
            Kind::AutoCfg(e) => fmt::Display::fmt(e, f),
            Kind::VersionCheck => f.write_str("version_check error"),
        }
    }
}

impl std::error::Error for Error
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        match &self.0 {
            Kind::AutoCfg(e) => Some(e),
            Kind::VersionCheck => None,
        }
    }
}
