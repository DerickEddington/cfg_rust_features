#![cfg_attr(
    // The "test" feature is currently unrecognized by the `cfg_rust_features` crate, and so this
    // demonstrates and exercises the design pattern of using `cfg` with such in anticipation of
    // the possibility of it becoming both recognized and stable.  This design pattern is
    // intentionally supported by the `cfg_rust_features` crate.
    all(not(rust_lib_feature = "test"), rust_comp_feature = "unstable_features"),
    // A nightly (or dev) compiler is being used and the feature is still unstable.
    feature(test)
)]
#![cfg_attr(
    special_dev_test = "enable-unstable-features",
    // For development testing, pretend that the recognized features have become stable.
    feature(
        arbitrary_self_types,
        cfg_version,
        destructuring_assignment,
        error_in_core,
        inner_deref,
        iter_zip,
        never_type,
        question_mark,
        step_trait,
        unwrap_infallible,
    )
)]

// Similar to above, this uses a currently-unrecognized feature.
#[cfg(any(rust_lib_feature = "test", rust_comp_feature = "unstable_features"))]
// Either: the feature has become stable, or a nightly (or dev) compiler is being used.
extern crate test;

#[cfg(test)]
mod tests
{
    #[allow(dead_code)]
    mod never_type_hack
    {
        pub type Never = <F as HasOutput>::Output;

        pub trait HasOutput
        {
            type Output;
        }

        impl<O> HasOutput for fn() -> O
        {
            type Output = O;
        }

        pub type F = fn() -> !;
    }

    #[cfg(rust_lang_feature = "arbitrary_self_types")]
    #[test]
    fn arbitrary_self_types()
    {
        struct Wrap<T>(T);

        impl<T> core::ops::Deref for Wrap<T>
        {
            type Target = T;

            fn deref(&self) -> &Self::Target
            {
                &self.0
            }
        }

        trait Trait
        {
            fn trait_method(self: Wrap<&Self>) -> &Self
            {
                &self.0
            }
        }

        struct Thing<T>(T);

        impl<T> Trait for Thing<T> {}

        impl<T: Copy> Thing<T>
        {
            fn inherent_method(self: &Wrap<Self>) -> T
            {
                (self.0).0
            }
        }

        assert!(Wrap(&Thing(true)).trait_method().0);
        assert!(Wrap(Thing(true)).inherent_method());
    }

    #[cfg(rust_lang_feature = "cfg_version")]
    #[test]
    fn cfg_version()
    {
        // Prevent old Rust versions from erroring on the attribute syntax.
        macro_rules! shield {
            () => {
                #[cfg(version("1.0.0"))]
                struct S;
                let _ = S;
            };
        }
        shield!();
    }

    #[cfg(rust_lang_feature = "destructuring_assignment")]
    #[test]
    fn destructuring_assignment()
    {
        let (a, b);
        (a, b) = (true, false);
        assert_ne!(a, b);
    }

    #[cfg(rust_lib_feature = "error_in_core")]
    #[test]
    fn error_in_core()
    {
        let e: &core::error::Error = &std::fmt::Error;
        assert!(e.is::<std::fmt::Error>());
    }

    #[cfg(rust_lib_feature = "inner_deref")]
    #[test]
    fn inner_deref()
    {
        assert_eq!(Ok(&1), Ok::<_, ()>(Box::new(1)).as_deref());
    }

    #[cfg(rust_lib_feature = "iter_zip")]
    #[test]
    fn iter_zip()
    {
        assert_eq!(vec![(1, 2)], std::iter::zip([1], [2]).collect::<Vec<_>>());
    }

    #[cfg(rust_lang_feature = "never_type")]
    #[test]
    fn never_type()
    {
        // Prevent old Rust versions from erroring on the `!` syntax.
        macro_rules! shield {
            () => {
                let _: [!; 0] = [];
            };
        }
        shield!();
    }

    #[cfg(rust_lang_feature = "question_mark")]
    #[test]
    fn question_mark()
    {
        // Prevent old Rust versions from erroring on the `?` syntax.
        macro_rules! shield {
            () => {
                Err(())?
            };
        }
        fn f() -> Result<(), ()>
        {
            shield!()
        }
        assert_eq!(Err(()), f());
    }

    #[cfg(rust_comp_feature = "rust1")]
    #[test]
    fn rust1_comp() {}

    #[cfg(rust_lang_feature = "rust1")]
    #[test]
    fn rust1_lang() {}

    #[cfg(rust_lib_feature = "rust1")]
    #[test]
    fn rust1_lib() {}

    #[cfg(rust_lib_feature = "step_trait")]
    #[test]
    fn step_trait()
    {
        use std::iter::Step;
        fn f<T: Step>(x: T) -> Option<T>
        {
            Step::forward_checked(x, 1)
        }
        assert_eq!(Some(2), f(1))
    }

    // Similar to above, this exercises using a `cfg` option that is currently unsupported by the
    // `cfg_rust_features` crate but that possibly could be supported in the future.
    #[cfg(rust_lib_feature = "test")]
    #[bench]
    fn test(_bencher: &mut test::Bencher) {}

    #[cfg(rust_comp_feature = "unstable_features")]
    #[test]
    fn unstable_features()
    {
        #![allow(dead_code)]
    }

    #[cfg(rust_lib_feature = "unwrap_infallible")]
    #[test]
    fn unwrap_infallible()
    {
        assert_eq!(1, Ok::<_, never_type_hack::Never>(1).into_ok());
    }

    // This exercises using a non-existent feature that both Rust and the `cfg_rust_features`
    // crate and will never support, and so this item should never be compiled.
    #[cfg(rust_comp_feature = "SubGenius_Bogusness")]
    #[test]
    fn SubGenius_Bogusness()
    {
        assert!(false);
    }
}
