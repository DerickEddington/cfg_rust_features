use std::io;
use std::path::{PathBuf, Path};
use std::env::temp_dir;
use std::fs::{create_dir, remove_dir_all};
use std::ffi::OsStr;


pub struct TempSubDir(PathBuf);

impl TempSubDir
{
    /// Create a temporary directory with a name that should be unique enough for the tests of the
    /// parent package.
    pub fn new(subname: &str) -> io::Result<Self>
    {
        const UNIQUE: &'static str = "5a3fa1c4b3ed363f48a23fc7c10c9691";
        let dir = temp_dir().join(format!("cfg_rust_features-{}-{}", subname, UNIQUE));
        create_dir(&dir).map(|()| TempSubDir(dir))
    }

    /// Removes the directory, after removing all its contents.
    pub fn delete_all(&mut self) -> io::Result<()>
    {
        remove_dir_all(&self.0)
    }
}

impl AsRef<Path> for TempSubDir
{
    fn as_ref(&self) -> &Path
    {
        &self.0
    }
}

impl AsRef<OsStr> for TempSubDir
{
    fn as_ref(&self) -> &OsStr
    {
        self.0.as_ref()
    }
}

impl Drop for TempSubDir
{
    fn drop(&mut self)
    {
        let _ = self.delete_all();
    }
}
