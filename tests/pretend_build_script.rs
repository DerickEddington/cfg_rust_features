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
        env,
        fs::File,
        io,
        path::Path,
    },
    tempfile::tempdir,
};


type PretendResult = Result<(), cfg_rust_features::Error>;
type MainResult = Result<(), Box<dyn std::error::Error>>;


fn with_redirected_stdout_stderr(
    dir: &Path,
    thunk: impl FnOnce() -> PretendResult,
) -> MainResult
{
    type StdoutStderr = (FileDescriptor, FileDescriptor);

    fn set_stdout_stderr<F: AsRawFileDescriptor>(
        stdout: &F,
        stderr: &F,
    ) -> Result<StdoutStderr, filedescriptor::Error>
    {
        let orig_stdout = FileDescriptor::redirect_stdio(stdout, StdioDescriptor::Stdout)?;
        let orig_stderr = FileDescriptor::redirect_stdio(stderr, StdioDescriptor::Stderr)?;
        Ok((orig_stdout, orig_stderr))
    }

    fn redirect_stdout_stderr(dir: &Path) -> Result<StdoutStderr, Box<dyn std::error::Error>>
    {
        fn create_file_in(
            dir: &Path,
            name: &str,
        ) -> io::Result<File>
        {
            File::create(dir.join(name))
        }

        let stdout_file = create_file_in(dir, "stdout")?;
        let stderr_file = create_file_in(dir, "stderr")?;
        Ok(set_stdout_stderr(&stdout_file, &stderr_file)?)
    }

    let (orig_stdout, orig_stderr) = redirect_stdout_stderr(dir)?;
    let result = thunk();
    set_stdout_stderr(&orig_stdout, &orig_stderr)?;
    Ok(result?)
}

fn pretend_build_script() -> PretendResult
{
    emit_rerun_if_changed_file(file!());

    CfgRustFeatures::new()?.emit_rust_features([
        "step_trait",
        "never_type",
        "unwrap_infallible",
        "unstable_features",
    ])?;

    Ok(())
}

fn show_captured_stdout_stderr(dir: &Path) -> MainResult
{
    use std::io::Write as _;

    // Accumulate what to show before actually showing it, in case an error happens before
    // finishing.
    let mut accum = Vec::new();
    let mut show = |name| -> MainResult {
        let mut captured = File::open(dir.join(name))?;
        writeln!(
            &mut accum,
            "=== Build-Script {} ========================================================",
            name,
        )?;
        io::copy(&mut captured, &mut accum)?;
        writeln!(
            &mut accum,
            "================================================================================"
        )?;
        Ok(())
    };

    show("stdout")?;
    show("stderr")?;
    // Now actually show it.
    io::copy(&mut &*accum, &mut io::stdout())?;
    Ok(())
}

fn main() -> MainResult
{
    // Setup to pretend that this program is a build script.
    let out_dir = tempdir()?;
    env::set_var("OUT_DIR", out_dir.path());

    let result = with_redirected_stdout_stderr(out_dir.path(), pretend_build_script);
    show_captured_stdout_stderr(out_dir.path())?;

    out_dir.close()?;
    result
}
