use {
    cfg_rust_features::{
        emit_rerun_if_changed_file,
        CfgRustFeatures,
        EnabledFeatures,
        FeatureCategory,
        FeatureName,
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
        hash::Hash,
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


/// Like a `main` function of a build script (modulo the `Ok` type).
fn pretend_build_script() -> ResultDynErr<EnabledFeatures<'static>>
{
    emit_rerun_if_changed_file(file!());

    Ok(CfgRustFeatures::new()?.emit_rust_features([
        "inner_deref",
        "iter_zip",
        "never_type",
        "step_trait",
        "unstable_features",
        "unwrap_infallible",
    ])?)
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
    assert_results(&call_result?, &captured_stdio);
    captured_stdio.delete()?;
    Ok(())
}


/// Must correspond to what [`pretend_build_script`] emits.
fn assert_results(
    call_result: &EnabledFeatures<'static>,
    captured_stdio: &CapturedStdio,
)
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

    let required_features = HashSet::from([
        // As required, because it is since 1.47, before our `package.rust-version`.  This
        // enables the testing that at least one feature is detected.
        Feature { category: "lib", name: "inner_deref" },
        // Feature { category: "lang", name: "rust1" }, // TODO: (or whatever the name is)
        // Feature { category: "lib",  name: "rust1" }, // TODO: (or whatever the name is)
    ]);
    let optional_features = HashSet::from([
        Feature { category: "comp", name: "unstable_features" },
        Feature { category: "lang", name: "never_type" },
        Feature { category: "lib", name: "iter_zip" },
        Feature { category: "lib", name: "step_trait" },
        Feature { category: "lib", name: "unwrap_infallible" },
    ]);
    let allowed_features =
        HashSet::from_iter(required_features.union(&optional_features).copied());

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
            hashset.iter().copied().map(|feat| (feat.name, feat.category)).collect()
        }

        let enabled = from_hashmap(call_result);
        let required = from_hashset(&required_features);
        let allowed = from_hashset(&allowed_features);

        assert_enabled_fits_required_and_allowed(enabled, required, allowed);
    }

    // Check the stdout lines, emitted by the call to CfgRustFeatures::emit_rust_features, which
    // instruct Cargo to set compilation parameters like the `cfg` predicates.
    {
        fn fmt_cargo_instructions(features: &HashSet<Feature>) -> Vec<String>
        {
            Vec::from_iter(features.iter().map(|feature| {
                format!("cargo:rustc-cfg=rust_{}_feature={:?}", feature.category, feature.name)
            }))
        }

        let lines: HashSet<String> = {
            let vec = Vec::from_iter(captured_stdio.out.lines().map(String::from));
            let set = HashSet::from_iter(vec.iter().cloned());
            assert_eq!(set.len(), vec.len()); // No duplicate lines.
            set
        };
        let required = HashSet::from_iter(
            [
                &[format!("cargo:rerun-if-changed={}", file!())][..],
                &fmt_cargo_instructions(&required_features),
            ]
            .concat(),
        );
        let optional = HashSet::from_iter(fmt_cargo_instructions(&optional_features));
        let allowed = HashSet::from_iter(required.union(&optional).cloned());

        assert_enabled_fits_required_and_allowed(lines, required, allowed);
    }
}
