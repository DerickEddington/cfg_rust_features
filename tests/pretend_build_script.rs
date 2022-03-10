use {
    cfg_rust_features::{
        emit_rerun_if_changed_file,
        CfgRustFeatures,
    },
    filedescriptor::{
        AsRawFileDescriptor,
        FileDescriptor,
        StdioDescriptor,
    },
    std::{
        collections::HashSet,
        env,
        error::Error,
        fs::File,
        io::{
            self,
            Read as _,
        },
        path::Path,
    },
    tempfile::{
        tempdir,
        TempDir,
    },
};


type ResultDynErr<T> = Result<T, Box<dyn Error>>;

#[non_exhaustive]
pub struct CapturedStdio
{
    /// Temporary directory where `stdout` and `stderr` are redirected into files.  Automatically
    /// deleted.
    dir:     TempDir,
    /// `stdout` contents
    pub out: String,
    /// `stderr` contents
    pub err: String,
}

impl CapturedStdio
{
    pub fn new() -> Result<Self, impl Error>
    {
        tempdir().map(|dir| Self { dir, out: String::new(), err: String::new() })
    }

    pub fn dir(&self) -> &Path
    {
        self.dir.path()
    }

    pub fn for_call<V>(
        &mut self,
        func: impl FnOnce() -> V,
    ) -> ResultDynErr<V>
    {
        type StdioFDs = (FileDescriptor, FileDescriptor);

        fn redirect_to<F: AsRawFileDescriptor>(
            stdout: &F,
            stderr: &F,
        ) -> Result<StdioFDs, filedescriptor::Error>
        {
            let orig_stdout = FileDescriptor::redirect_stdio(stdout, StdioDescriptor::Stdout)?;
            let orig_stderr = FileDescriptor::redirect_stdio(stderr, StdioDescriptor::Stderr)?;
            Ok((orig_stdout, orig_stderr))
        }

        fn redirect_in(dir: &Path) -> ResultDynErr<StdioFDs>
        {
            let create_file_in = |name| File::create(dir.join(name));
            let stdout_file = create_file_in("stdout")?;
            let stderr_file = create_file_in("stderr")?;
            Ok(redirect_to(&stdout_file, &stderr_file)?)
        }

        // Temporarily redirect `stdout` and `stderr` to files in our temporary directory.
        let (orig_stdout, orig_stderr) = redirect_in(self.dir())?;
        // Call the given `thunk` with the redirection in effect.
        let value = func();
        // Revert the redirection.
        redirect_to(&orig_stdout, &orig_stderr)?;
        // Read and keep the captured outputs from the redirection.
        self.load_captured()?;
        Ok(value)
    }

    fn load_captured(&mut self) -> io::Result<()>
    {
        let load_to =
            |name, string| File::open(self.dir.path().join(name))?.read_to_string(string);
        load_to("stdout", &mut self.out)?;
        load_to("stderr", &mut self.err)?;
        Ok(())
    }

    pub fn show(
        &self,
        title: &str,
    )
    {
        let print_delim = |name, contents| {
            println!(
                "
=== {} {} ========================================================
{}=== end {} =================================================================",
                title, name, contents, name
            )
        };

        print_delim("stdout", &self.out);
        print_delim("stderr", &self.err);
    }

    pub fn delete(self) -> Result<(), impl Error>
    {
        self.dir.close()
    }
}


/// Exactly like a `main` function of a build script could be.
fn pretend_build_script() -> ResultDynErr<()>
{
    emit_rerun_if_changed_file(file!());

    CfgRustFeatures::new()?.emit_rust_features([
        "inner_deref",
        "iter_zip",
        "never_type",
        "step_trait",
        "unstable_features",
        "unwrap_infallible",
    ])?;

    Ok(())
}


/// Enables having our own extra methods on [`CapturedStdio`].
trait Asserter
{
    /// Must correspond to what [`pretend_build_script`] emits.
    ///
    /// Only checks the captured `stdout` contents, because only that is used by Cargo with build
    /// scripts.
    fn assert(&self);
}

impl Asserter for CapturedStdio
{
    fn assert(&self)
    {
        let lines: HashSet<&str> = {
            let vec = Vec::from_iter(self.out.lines());
            let set = HashSet::from_iter(vec.iter().copied());
            assert_eq!(set.len(), vec.len()); // No duplicate lines.
            set
        };
        let rerun_if_changed = format!("cargo:rerun-if-changed={}", file!());
        let required = HashSet::from([
            &*rerun_if_changed,
            // As required, because it is since 1.47, before our `package.rust-version`.  This
            // enables the testing that at least one feature is detected.
            r#"cargo:rustc-cfg=rust_lib_feature="inner_deref""#,
        ]);
        let optional = HashSet::from([
            r#"cargo:rustc-cfg=rust_lib_feature="iter_zip""#,
            r#"cargo:rustc-cfg=rust_lang_feature="never_type""#,
            r#"cargo:rustc-cfg=rust_lib_feature="step_trait""#,
            r#"cargo:rustc-cfg=rust_lib_feature="unwrap_infallible""#,
            r#"cargo:rustc-cfg=rust_comp_feature="unstable_features""#,
        ]);
        let allowed = HashSet::from_iter(required.union(&optional).copied());

        assert!(lines.is_superset(&required));
        assert!(lines.is_subset(&allowed));
    }
}


fn main() -> ResultDynErr<()>
{
    let has_opt = {
        let args = Vec::from_iter(env::args().skip(1));
        move |opt| args.contains(&format!("--{}", opt))
    };

    // Setup to pretend that this program is a build script.
    let mut captured_stdio = CapturedStdio::new()?;
    env::set_var("OUT_DIR", captured_stdio.dir());

    let call_result = captured_stdio.for_call(pretend_build_script)?;
    if has_opt("show-output") {
        captured_stdio.show("build-script");
    }
    captured_stdio.assert();
    captured_stdio.delete()?;
    call_result
}
