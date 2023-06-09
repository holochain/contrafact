use std::marker::PhantomData;

use arbitrary::Unstructured;
use lens_rs::*;

use crate::{fact::Bounds, Fact, *};

pub fn optical<'a, Src, Img, Optics, F, L>(
    label: L,
    optics: Optics,
    inner_fact: F,
) -> OpticalFact<'a, Src, Img, Optics, F>
where
    Src: Bounds<'a> + Lens<Optics, Img>,
    Img: Bounds<'a> + Clone,
    Optics: Clone + std::fmt::Debug,
    F: Fact<'a, Img>,
    L: ToString,
{
    OpticalFact::new(label.to_string(), optics, inner_fact)
}

/// A fact which uses a lens to apply another fact. Use [`lens()`] to construct.
#[derive(Clone)]
pub struct OpticalFact<'a, Src, Img, Optics, F>
where
    Src: Bounds<'a> + Lens<Optics, Img>,
    Img: Bounds<'a> + Clone,
    Optics: Clone + std::fmt::Debug,
    F: Fact<'a, Img>,
{
    label: String,

    optics: Optics,

    /// The inner_fact about the inner substructure
    inner_fact: F,

    __phantom: PhantomData<&'a (Src, Img)>,
}

impl<'a, Src, Img, Optics, F> OpticalFact<'a, Src, Img, Optics, F>
where
    Src: Bounds<'a> + Lens<Optics, Img>,
    Img: Bounds<'a> + Clone,
    Optics: Clone + std::fmt::Debug,
    F: Fact<'a, Img>,
{
    pub fn new(label: String, optics: Optics, inner_fact: F) -> Self {
        Self {
            label,
            optics,
            inner_fact,
            __phantom: PhantomData::<&(Src, Img)>,
        }
    }
}

impl<'a, Src, Img, Optics, F> Fact<'a, Src> for OpticalFact<'a, Src, Img, Optics, F>
where
    Src: Bounds<'a> + Lens<Optics, Img>,
    Img: Bounds<'a> + Clone,
    Optics: Clone + std::fmt::Debug,
    F: Fact<'a, Img>,
{
    // TODO: remove
    #[tracing::instrument(skip(self))]
    fn check(&self, obj: &Src) -> Check {
        let imgs = obj.traverse_ref(self.optics.clone());
        imgs.iter()
            .enumerate()
            .flat_map(|(i, img)| {
                let label = if imgs.len() > 1 {
                    format!("{}[{}]", self.label, i)
                } else {
                    self.label.clone()
                };

                self.inner_fact
                    .check(img)
                    .map(|err| format!("lens({}){{{:?}}} > {}", label, self.optics.clone(), err))
            })
            .collect::<Vec<_>>()
            .into()
    }

    #[tracing::instrument(skip(self, g))]
    #[cfg(feature = "mutate-inplace")]
    fn mutate(&self, obj: &mut Src, g: &mut Generator<'a>) {
        let t = obj.view_mut(self.optics.clone());
        self.inner_fact.mutate(t, g);
    }

    fn mutate(&self, mut obj: Src, g: &mut Generator<'a>) -> Src {
        for img in obj.traverse_mut(self.optics.clone()) {
            *img = self.inner_fact.mutate(img.clone(), g);
        }
        obj
    }

    #[tracing::instrument(skip(self))]
    fn advance(&mut self, obj: &Src) {
        for img in obj.traverse_ref(self.optics.clone()) {
            self.inner_fact.advance(img);
        }
    }
}

#[test]
fn test_lens() {
    let x = (1u8, (2u8, (3u8, 4u8)));

    let mut fact = OpticalFact {
        label: "".into(),
        optics: optics!(_1._1._1),
        inner_fact: eq_(3),
        __phantom: PhantomData::<&((u8, (u8, (u8, u8))), u8)>,
    };

    assert_eq!(fact.check(&x).errors().len(), 1);

    fact.inner_fact = eq_(4);
    assert!(fact.check(&x).is_ok());
}
