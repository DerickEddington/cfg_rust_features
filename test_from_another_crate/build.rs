#[macro_use(emit)]
extern crate cfg_rust_features;

fn main()
{
    emit!(vec![
        "arbitrary_self_types",
        "cfg_version",
        "destructuring_assignment",
        "inner_deref",
        "iter_zip",
        "never_type",
        "question_mark",
        "rust1",
        "step_trait",
        "unstable_features",
        "unwrap_infallible",
    ])
    .unwrap();
}
