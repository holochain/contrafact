use arbitrary::Arbitrary;
use contrafact::*;

type Id = u32;

// Similar to Holochain's DhtOp
#[derive(Clone, Debug, PartialEq, Arbitrary)]
enum Omega {
    AlphaBeta { id: Id, alpha: Alpha, beta: Beta },
    Alpha { id: Id, alpha: Alpha },
}

impl Omega {
    fn alpha(&self) -> &Alpha {
        match self {
            Self::AlphaBeta { alpha, .. } => alpha,
            Self::Alpha { alpha, .. } => alpha,
        }
    }

    fn alpha_mut(&mut self) -> &mut Alpha {
        match self {
            Self::AlphaBeta { alpha, .. } => alpha,
            Self::Alpha { alpha, .. } => alpha,
        }
    }

    fn _beta(&self) -> Option<&Beta> {
        match self {
            Self::AlphaBeta { beta, .. } => Some(beta),
            Self::Alpha { .. } => None,
        }
    }

    fn beta_mut(&mut self) -> Option<&mut Beta> {
        match self {
            Self::AlphaBeta { beta, .. } => Some(beta),
            Self::Alpha { .. } => None,
        }
    }

    fn pi(&self) -> Pi {
        match self.clone() {
            Self::AlphaBeta { alpha, beta, .. } => Pi(alpha, Some(beta)),
            Self::Alpha { alpha, .. } => Pi(alpha, None),
        }
    }

    fn id(&self) -> &Id {
        match self {
            Self::AlphaBeta { id, .. } => id,
            Self::Alpha { id, .. } => id,
        }
    }

    fn id_mut(&mut self) -> &mut Id {
        match self {
            Self::AlphaBeta { id, .. } => id,
            Self::Alpha { id, .. } => id,
        }
    }
}

// Similar to Holochain's Action
#[derive(Clone, Debug, PartialEq, Arbitrary)]
enum Alpha {
    Beta { id: Id, beta: Beta, data: String },
    Nil { id: Id, data: String },
}

impl Alpha {
    fn id(&mut self) -> &mut Id {
        match self {
            Self::Beta { id, .. } => id,
            Self::Nil { id, .. } => id,
        }
    }
    fn data(&mut self) -> &mut String {
        match self {
            Self::Beta { data, .. } => data,
            Self::Nil { data, .. } => data,
        }
    }
}

// Similar to Holochain's Entry
#[derive(Clone, Debug, PartialEq, Arbitrary)]
struct Beta {
    id: u32,
    data: String,
}

#[derive(Clone, Debug, PartialEq, Arbitrary)]
/// Similar to Holochain's SignedActionHashed
struct Sigma {
    alpha: Alpha,
    id2: Id,
    sig: String,
}

#[derive(Clone, Debug, PartialEq, Arbitrary)]
/// Similar to Holochain's Record
struct Rho {
    sigma: Sigma,
    beta: Option<Beta>,
}

/// Some struct needed to set the values of a Sigma whenever its Alpha changes.
/// Analogous to Holochain's Keystore (MetaLairClient).
struct AlphaSigner;

impl AlphaSigner {
    fn sign(&self, mut alpha: Alpha) -> Sigma {
        Sigma {
            id2: alpha.id().clone() * 2,
            sig: alpha.id().to_string(),
            alpha,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Arbitrary)]
struct Pi(Alpha, Option<Beta>);

fn pi_beta_match() -> Facts<'static, Pi> {
    facts![brute(
        "Pi alpha has matching beta iff beta is Some",
        |p: &Pi| match p {
            Pi(Alpha::Beta { beta, .. }, Some(b)) => beta == b,
            Pi(Alpha::Nil { .. }, None) => true,
            _ => false,
        }
    )]
}

/// - All data must be set as specified
/// - All Ids should match each other. If there is a Beta, its id should match too.
fn pi_fact(id: Id, data: String) -> Facts<'static, Pi> {
    let alpha_fact = facts![
        lens("Alpha::id", |a: &mut Alpha| a.id(), eq("id", id)),
        lens("Alpha::data", |a: &mut Alpha| a.data(), eq("data", data)),
    ];
    let beta_fact = lens("Beta::id", |b: &mut Beta| &mut b.id, eq("id", id));
    facts![
        pi_beta_match(),
        lens("Pi::alpha", |o: &mut Pi| &mut o.0, alpha_fact),
        prism("Pi::beta", |o: &mut Pi| o.1.as_mut(), beta_fact),
    ]
}

/// - All Ids should match each other. If there is a Beta, its id should match too
/// - If Omega::Alpha,     then Alpha::Nil.
/// - If Omega::AlphaBeta, then Alpha::Beta,
///     - and, the the Betas of the Alpha and the Omega should match.
/// - all data must be set as specified
fn omega_fact(id: Id, data: String) -> Facts<'static, Omega> {
    let omega_pi = LensFact::new(
        "Omega -> Pi",
        |o| match o {
            Omega::AlphaBeta { alpha, beta, .. } => Pi(alpha, Some(beta)),
            Omega::Alpha { alpha, .. } => Pi(alpha, None),
        },
        |o, pi| {
            let id = o.id().clone();
            match pi {
                Pi(alpha, Some(beta)) => Omega::AlphaBeta { id, alpha, beta },
                Pi(alpha, None) => Omega::Alpha { id, alpha },
            }
        },
        pi_fact(id, data),
    );

    facts![
        omega_pi,
        lens("Omega::id", |o: &mut Omega| o.id_mut(), eq("id", id)),
    ]
}

/// TODO: use me
fn rho_fact(id: Id, data: String, signer: AlphaSigner) -> Facts<'static, Rho> {
    let rho_pi = LensFact::new(
        "Rho -> Pi",
        |rho: Rho| Pi(rho.sigma.alpha, rho.beta),
        move |mut rho, Pi(a, b)| {
            rho.sigma = signer.sign(a);
            rho.beta = b;
            rho
        },
        pi_fact(id, data),
    );
    facts![rho_pi]
}

#[test]
fn test_omega_fact() {
    observability::test_run().ok();
    let mut u = utils::unstructured_noise();

    let fact = omega_fact(11, "spartacus".into());

    let beta = Beta::arbitrary(&mut u).unwrap();

    let mut valid1 = Omega::Alpha {
        id: 8,
        alpha: Alpha::Nil {
            id: 3,
            data: "cheese".into(),
        },
    };

    let mut valid2 = Omega::AlphaBeta {
        id: 8,
        alpha: Alpha::Nil {
            id: 3,
            data: "cheese".into(),
        },
        beta: beta.clone(),
    };

    valid1 = fact.mutate(valid1, &mut u);
    fact.check(dbg!(&valid1)).unwrap();

    valid2 = fact.mutate(valid2, &mut u);
    fact.check(dbg!(&valid2)).unwrap();

    let mut invalid1 = Omega::Alpha {
        id: 8,
        alpha: Alpha::Beta {
            id: 3,
            data: "cheese".into(),
            beta: beta.clone(),
        },
    };

    let mut invalid2 = Omega::AlphaBeta {
        id: 8,
        alpha: Alpha::Nil {
            id: 3,
            data: "cheese".into(),
        },
        beta: beta.clone(),
    };

    // Ensure that check fails for invalid data
    assert_eq!(
        dbg!(fact.check(dbg!(&invalid1)).result().unwrap_err()).len(),
        4,
    );
    invalid1 = fact.mutate(invalid1, &mut u);
    fact.check(dbg!(&invalid1)).unwrap();

    // Ensure that check fails for invalid data
    assert_eq!(
        dbg!(fact.check(dbg!(&invalid2)).result().unwrap_err()).len(),
        5,
    );
    invalid2 = fact.mutate(invalid2, &mut u);
    fact.check(dbg!(&invalid2)).unwrap();
}
