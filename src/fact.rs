use arbitrary::*;

use crate::Check;

pub(crate) const SATISFY_ATTEMPTS: usize = 10;

/// The trait bounds for the subject of a Fact
pub trait Bounds: std::fmt::Debug + PartialEq + Arbitrary<'static> + Clone {}
impl<T> Bounds for T where T: std::fmt::Debug + PartialEq + Arbitrary<'static> + Clone {}

/// Type alias for a boxed Fact
pub type BoxFact<'a, T> = Box<dyn 'a + Fact<T>>;

/// Type alias for a Vec of boxed Facts
pub type Facts<'a, T> = Vec<BoxFact<'a, T>>;

/// A declarative representation of a constraint on some data, which can be
/// used to both make an assertion (check) or to mold some arbitrary existing
/// data into a shape which passes that same assertion (mutate)
pub trait Fact<T>
where
    T: Bounds,
{
    /// Assert that the constraint is satisfied (panic if not).
    fn check(&mut self, obj: &T) -> Check;

    /// Apply a mutation which moves the obj closer to satisfying the overall
    /// constraint.
    fn mutate(&mut self, obj: &mut T, u: &mut Unstructured<'static>);

    /// Mutate a value such that it satisfies the constraint.
    /// If the constraint cannot be satisfied, panic.
    fn satisfy(&mut self, obj: &mut T, u: &mut Unstructured<'static>) {
        let mut last_failure: Vec<String> = vec![];
        for _i in 0..SATISFY_ATTEMPTS {
            self.mutate(obj, u);
            if let Err(errs) = self.check(obj).ok() {
                last_failure = errs;
            } else {
                return;
            }
        }
        panic!(format!(
            "Could not satisfy a constraint even after {} iterations. Last check failure: {:?}",
            SATISFY_ATTEMPTS, last_failure
        ));
    }

    /// Build a new value such that it satisfies the constraint
    fn build(&mut self, u: &mut Unstructured<'static>) -> T {
        let mut obj = T::arbitrary(u).unwrap();
        self.satisfy(&mut obj, u);
        obj
    }
}

// pub trait Fact2<A, B>
// where
//     A: Bounds,
//     B: Bounds,
// {
//     /// Assert that the constraint is satisfied (panic if not).
//     fn check(&mut self, obj: (&A, &B)) -> Check;

//     /// Apply a mutation which moves the obj closer to satisfying the overall
//     /// constraint.
//     fn mutate(&mut self, obj: (&mut A, &mut B), u: &mut Unstructured<'static>);

//     /// Mutate a value such that it satisfies the constraint.
//     /// If the constraint cannot be satisfied, panic.
//     fn satisfy(&mut self, obj: (&mut A, &mut B), u: &mut Unstructured<'static>) {
//         let mut last_failure: Vec<String> = vec![];
//         for _i in 0..SATISFY_ATTEMPTS {
//             self.mutate(obj, u);
//             if let Err(errs) = self.check(obj).ok() {
//                 last_failure = errs;
//             } else {
//                 return;
//             }
//         }
//         panic!(format!(
//             "Could not satisfy a constraint even after {} iterations. Last check failure: {:?}",
//             SATISFY_ATTEMPTS, last_failure
//         ));
//     }

//     /// Build a new value such that it satisfies the constraint
//     fn build(&mut self, u: &mut Unstructured<'static>) -> A {
//         let mut obj = A::arbitrary(u).unwrap();
//         self.satisfy(&mut obj, u);
//         obj
//     }
// }

impl<T, F> Fact<T> for Box<F>
where
    T: Bounds,
    F: Fact<T> + ?Sized,
{
    #[tracing::instrument(skip(self))]
    fn check(&mut self, obj: &T) -> Check {
        tracing::trace!("check");
        (*self).as_mut().check(obj)
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&mut self, obj: &mut T, u: &mut Unstructured<'static>) {
        (*self).as_mut().mutate(obj, u);
    }
}

impl<T, F> Fact<T> for &mut [F]
where
    T: Bounds,
    F: Fact<T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&mut self, obj: &T) -> Check {
        self.iter_mut()
            .flat_map(|f| f.check(obj))
            .collect::<Vec<_>>()
            .into()
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&mut self, obj: &mut T, u: &mut Unstructured<'static>) {
        for f in self.iter_mut() {
            f.mutate(obj, u)
        }
    }
}

impl<T, F> Fact<T> for Vec<F>
where
    T: Bounds,
    F: Fact<T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&mut self, obj: &T) -> Check {
        self.as_mut_slice().check(obj)
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&mut self, obj: &mut T, u: &mut Unstructured<'static>) {
        self.as_mut_slice().mutate(obj, u)
    }
}
