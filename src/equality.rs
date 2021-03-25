use crate::*;
use egg::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(from = "SerializedEq")]
#[serde(into = "SerializedEq")]
#[serde(bound = "L: SynthLanguage")]
pub struct Equality<L: SynthLanguage> {
    pub name: Rc<str>,
    pub lhs: Pattern<L>,
    pub ids: Option<(Id, Id)>,
    pub rhs: Pattern<L>,
    // pub cond: Option<Pattern<L>>,
    pub rewrites: Vec<Rewrite<L, SynthAnalysis>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SerializedEq {
    lhs: String,
    rhs: String,
    bidirectional: bool,
}

impl<L: SynthLanguage + 'static> From<SerializedEq> for Equality<L> {
    fn from(ser: SerializedEq) -> Self {
        let lhs: Pattern<L> = ser.lhs.parse().unwrap();
        let rhs: Pattern<L> = ser.rhs.parse().unwrap();
        let mut rewrites =
            vec![Rewrite::new(format!("{} => {}", lhs, rhs), lhs.clone(), rhs.clone()).unwrap()];
        let name = if ser.bidirectional {
            rewrites.push(
                Rewrite::new(format!("{} => {}", rhs, lhs), rhs.clone(), lhs.clone()).unwrap(),
            );
            format!("{} <=> {}", lhs, rhs)
        } else {
            format!("{} => {}", lhs, rhs)
        };
        Self {
            name: name.into(),
            lhs,
            rhs,
            rewrites,
            ids: None,
        }
    }
}

impl<L: SynthLanguage> From<Equality<L>> for SerializedEq {
    fn from(eq: Equality<L>) -> Self {
        Self {
            lhs: eq.lhs.to_string(),
            rhs: eq.rhs.to_string(),
            bidirectional: eq.rewrites.len() > 1,
        }
    }
}

impl<L: SynthLanguage> Hash for Equality<L> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<L: SynthLanguage> Display for Equality<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<L: SynthLanguage> PartialEq for Equality<L> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<L: SynthLanguage> Eq for Equality<L> {}

struct NotUndefined<L: SynthLanguage> {
    name: String,
    rhs: Pattern<L>,
}

impl<L: SynthLanguage> Applier<L, SynthAnalysis> for NotUndefined<L> {
    fn vars(&self) -> Vec<Var> {
        self.rhs.vars()
    }

    fn apply_one(&self, egraph: &mut EGraph<L, SynthAnalysis>, matched_id: Id, subst: &Subst) -> Vec<Id> {
        if !egraph[matched_id].data.is_defined() {
            return vec![];
        }
        let ids = self.rhs.apply_one(egraph, matched_id, subst);
        assert_eq!(ids.len(), 1);
        if !egraph[ids[0]].data.is_defined() {
            return vec![]
        }


        for (a, b) in egraph[matched_id].data.cvec.iter().zip(&egraph[ids[0]].data.cvec) {
            match (a, b) {
                (Some(a), Some(b)) => assert_eq!(a, b, "bad rule {}", self.name),
                _ => (),
            }
        }

        ids
    }
}


impl<L: SynthLanguage> Equality<L> {
    pub fn new(e1: &RecExpr<L>, e2: &RecExpr<L>) -> Option<Self> {
        let mut forward: (String, Pattern<L>, Pattern<L>, Option<Rewrite<L, _>>) = {
            let map = &mut HashMap::default();
            let lhs = L::generalize(&e1, map);
            let rhs = L::generalize(&e2, map);
            let name = format!("{} => {}", lhs, rhs);
            let defined_rhs = NotUndefined { name: name.clone(), rhs: rhs.clone() };
            (
                name.clone(),
                lhs.clone(),
                rhs.clone(),
                Rewrite::new(name, lhs.clone(), defined_rhs).ok(),
            )
        };

        let mut back: (String, Pattern<L>, Pattern<L>, Option<Rewrite<L, _>>) = {
            let map = &mut HashMap::default();
            let lhs = L::generalize(&e2, map);
            let rhs = L::generalize(&e1, map);
            let name = format!("{} => {}", lhs, rhs);
            let defined_rhs = NotUndefined {name: name.clone(), rhs: rhs.clone() };
            (
                name.clone(),
                lhs.clone(),
                rhs.clone(),
                Rewrite::new(name, lhs.clone(), defined_rhs).ok(),
            )
        };

        // make sure we always do things in the same order
        if back.0 > forward.0 {
            std::mem::swap(&mut forward, &mut back);
        }

        match (forward, back) {
            ((_, _, _, None), (_, _, _, None)) => None,
            ((name, lhs, rhs, Some(rw)), (_, _, _, None))
            | ((_, _, _, None), (name, lhs, rhs, Some(rw))) => Some(Self {
                name: name.into(),
                lhs,
                rhs,
                ids: None,
                rewrites: vec![rw],
            }),
            ((_, lhs, rhs, Some(rw1)), (_, _, _, Some(rw2))) => Some(Self {
                name: format!("{} <=> {}", lhs, rhs).into(),
                lhs,
                rhs,
                ids: None,
                rewrites: if rw1.name() == rw2.name() {
                    vec![rw1]
                } else {
                    vec![rw1, rw2]
                },
            }),
        }
    }

    pub fn score(&self) -> [i32; 3] {
        L::score(&self.lhs, &self.rhs)
    }
}
