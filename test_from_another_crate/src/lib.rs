#![cfg_attr(rust_comp_feature = "unstable_features", feature(test))]
#[cfg(rust_comp_feature = "unstable_features")]
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
        // Old Rust versions would error if a non-identifier were used in the `version` form (like
        // the feature's RFC 2523 specifies), so we leave it empty so old versions do not error.
        // Hopefully this empty form will become an error if the feature is stabilized in future
        // versions, so that we can reevaluate how this test case should be based on the final
        // stabilized specification of the feature.
        #[cfg(version())]
        struct S;
        let _ = S;
    }

    #[cfg(rust_lang_feature = "destructuring_assignment")]
    #[test]
    fn destructuring_assignment()
    {
        let (a, b);
        (a, b) = (true, false);
        assert_ne!(a, b);
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

    #[cfg(rust_comp_feature = "unstable_features")]
    #[bench]
    fn unstable_features(_bencher: &mut test::Bencher) {}

    #[cfg(rust_lib_feature = "unwrap_infallible")]
    #[test]
    fn unwrap_infallible()
    {
        assert_eq!(1, Ok::<_, never_type_hack::Never>(1).into_ok());
    }
}
