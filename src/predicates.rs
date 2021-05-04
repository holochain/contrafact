//! Some predicates borrowed from predicates-rs
//! https://github.com/assert-rs/predicates-rs

use std::marker::PhantomData;

use crate::constraint::*;

pub fn eq<T>(constant: T) -> EqFact<T>
where
    T: std::fmt::Debug + PartialEq,
{
    EqFact {
        constant,
        op: EqOp::Equal,
    }
}

pub fn ne<T>(constant: T) -> EqFact<T>
where
    T: std::fmt::Debug + PartialEq,
{
    EqFact {
        constant,
        op: EqOp::NotEqual,
    }
}

pub fn in_iter<I, T>(iter: I) -> InFact<T>
where
    T: PartialEq + std::fmt::Debug,
    I: IntoIterator<Item = T>,
{
    use std::iter::FromIterator;
    InFact {
        inner: Vec::from_iter(iter),
    }
}

pub fn or<A, B, Item>(a: A, b: B) -> OrFact<A, B, Item>
where
    A: Constraint<Item>,
    B: Constraint<Item>,
    Item: Bounds,
{
    OrFact {
        a,
        b,
        _phantom: PhantomData,
    }
}

pub struct EqFact<T> {
    op: EqOp,
    constant: T,
}

pub enum EqOp {
    Equal,
    NotEqual,
}

impl<T> Constraint<T> for EqFact<T>
where
    T: Bounds + PartialEq,
{
    fn check(&self, obj: &T) {
        match self.op {
            EqOp::Equal => assert!(*obj == self.constant),
            EqOp::NotEqual => assert!(*obj != self.constant),
        }
        self.check(obj)
    }

    fn mutate(&mut self, obj: &mut T, u: &mut arbitrary::Unstructured<'static>) {
        match self.op {
            EqOp::Equal => *obj = self.constant.clone(),
            EqOp::NotEqual => loop {
                *obj = T::arbitrary(u).unwrap();
                if *obj != self.constant {
                    break;
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InFact<T>
where
    T: PartialEq + std::fmt::Debug,
{
    inner: Vec<T>,
}

impl<T> Constraint<T> for InFact<T>
where
    T: Bounds,
{
    fn check(&self, obj: &T) {
        assert!(self.inner.contains(obj))
    }

    fn mutate(&mut self, obj: &mut T, u: &mut arbitrary::Unstructured<'static>) {
        *obj = u.choose(self.inner.as_slice()).unwrap().clone();
        self.check(obj);
    }
}

/// Constraint that combines two `Constraint`s, returning the OR of the results.
///
/// This is created by the `or` function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrFact<M1, M2, Item>
where
    M1: Constraint<Item>,
    M2: Constraint<Item>,
    Item: ?Sized + Bounds,
{
    pub(crate) a: M1,
    pub(crate) b: M2,
    _phantom: PhantomData<Item>,
}

impl<P1, P2, T> Constraint<T> for OrFact<P1, P2, T>
where
    P1: Constraint<T> + Constraint<T>,
    P2: Constraint<T> + Constraint<T>,
    T: Bounds,
{
    fn check(&self, obj: &T) {
        todo!()
    }

    fn mutate(&mut self, obj: &mut T, u: &mut arbitrary::Unstructured<'static>) {
        if *u.choose(&[true, false]).unwrap() {
            self.a.mutate(obj, u);
        } else {
            self.b.mutate(obj, u);
        }
        self.check(obj);
    }
}
