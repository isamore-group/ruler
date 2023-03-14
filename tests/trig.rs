use num::rational::Ratio;
use num::BigInt;
use num::{Signed, Zero};
use ruler::enumo::{Ruleset, Workload};
use ruler::*;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::time::Instant;

pub type Rational = Ratio<BigInt>;

// custom implementation of real value
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Real(Symbol);

impl Real {
    pub fn as_str(self) -> &'static str {
        self.0.as_str()
    }
}

impl<S: AsRef<str>> From<S> for Real {
    fn from(s: S) -> Self {
        Real(Symbol::from(s.as_ref()))
    }
}

impl From<Real> for &'static str {
    fn from(s: Real) -> Self {
        s.as_str()
    }
}

impl FromStr for Real {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_empty() && s.chars().all(|c| c.is_numeric() || c == '-' || c == '/') {
            Ok(s.into())
        } else {
            Err("not real")
        }
    }
}

impl Display for Real {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl Debug for Real {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

// custom implementation of a complex value
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable(Symbol);

impl Variable {
    fn as_str(self) -> &'static str {
        self.0.as_str()
    }
}

impl<S: AsRef<str>> From<S> for Variable {
    fn from(s: S) -> Self {
        Variable(Symbol::from(s.as_ref()))
    }
}

impl From<Variable> for &'static str {
    fn from(s: Variable) -> Self {
        s.as_str()
    }
}

impl FromStr for Variable {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 1 && s.chars().next().unwrap().is_alphabetic() {
            Ok(s.into())
        } else if s.len() == 2 && s.starts_with('?') && s.chars().nth(1).unwrap().is_alphabetic() {
            Ok((&s[1..2]).into())
        } else {
            Err("not variable")
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl Debug for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

fn extract_constant(nodes: &[Trig]) -> Option<Rational> {
    for n in nodes {
        if let Trig::RealConst(v) = n {
            if let Ok(r) = v.as_str().parse() {
                return Some(r);
            }
        }
    }

    None
}

egg::define_language! {
  pub enum Trig {
    // trig operators
    "sin" = Sin(Id),
    "cos" = Cos(Id),
    "tan" = Tan(Id),
    // "csc" = Csc(Id),
    // "sec" = Sec(Id),
    // "cot" = Cot(Id),

    // complex exponetial
    "cis" = Cis(Id),

    // arithmetic operators
    "~" = Neg(Id),
    "+" = Add([Id; 2]),
    "-" = Sub([Id; 2]),
    "*" = Mul([Id; 2]),
    "/" = Div([Id; 2]),
    "sqr" = Sqr(Id),

    // constants
    "I" = Imag,
    "PI" = Pi,
    RealConst(Real),
    Var(Variable),
  }
}

impl SynthLanguage for Trig {
    type Constant = Real;

    fn is_rule_lifting() -> bool {
        true
    }

    fn get_lifting_rules() -> Ruleset<Self> {
        Ruleset::new(&[
            // definition of sine, cosine, tangent
            // (sine)
            "(sin ?a) ==> (/ (- (cis ?a) (cis (~ ?a))) (* 2 I))",
            "(/ (- (cis ?a) (cis (~ ?a))) (* 2 I)) ==> (sin ?a)",
            // (sine, alternatively)
            "(sin ?a) ==> (/ (- (* I (cis (~ ?a))) (* I (cis ?a))) 2)",
            "(/ (- (* I (cis (~ ?a))) (* I (cis ?a))) 2) => (sin ?a)",
            // (cosine)
            "(cos ?a) ==> (/ (+ (cis ?a) (cis (~ ?a))) 2)",
            "(/ (+ (cis ?a) (cis (~ ?a))) 2) ==> (cos ?a)",
            // (cosine, alternatively)
            "(cos ?a) ==> (/ (+ (* I (cis ?a)) (* I (cis (~ ?a)))) (* 2 I))",
            "(/ (+ (* I (cis ?a)) (* I (cis (~ ?a)))) (* 2 I)) ==> (cos ?a)",
            // (tangent, alternatively)
            "(tan ?a) ==> (* I (/ (- (cis (~ ?a)) (cis ?a)) (+ (cis (~ ?a)) (cis ?a))))",
            "(* I (/ (- (cis (~ ?a)) (cis ?a)) (+ (cis (~ ?a)) (cis ?a)))) ==> (tan ?a)",
            // relating tangent to sine and cosine
            "(tan ?a) ==> (/ (sin ?a) (cos ?a))",
            "(/ (sin ?a) (cos ?a)) ==> (tan ?a)",
            // definition of cos^2(a) and sin^2(a)
            "(* (cos ?a) (cos ?a)) ==> (/ (+ (+ (sqr (cis ?a)) (sqr (cis (~ ?a)))) 2) 4)",
            "(/ (+ (+ (sqr (cis ?a)) (sqr (cis (~ ?a)))) 2) 4) ==> (* (cos ?a) (cos ?a))",
            "(* (sin ?a) (sin ?a)) ==> (~ (/ (- (+ (sqr (cis ?a)) (sqr (cis (~ ?a)))) 2) 4))",
            "(~ (/ (- (+ (sqr (cis ?a)) (sqr (cis (~ ?a)))) 2) 4)) ==> (* (sin ?a) (sin ?a))",
            // definition of cos(a)*cos(b) and sin(a)*sin(b)
            "(* (cos ?x) (cos ?y)) ==> (/ (+ (+ (cis (- ?x ?y)) (cis (~ (- ?x ?y)))) (+ (cis (+ ?x ?y)) (cis (~ (+ ?x ?y))))) 4)",
            "(* (sin ?x) (sin ?y)) ==> (/ (- (+ (cis (- ?x ?y)) (cis (~ (- ?x ?y)))) (+ (cis (+ ?x ?y)) (cis (~ (+ ?x ?y))))) 4)",
            // definition of square
            "(sqr ?a) ==> (* ?a ?a)",
            "(* ?a ?a) ==> (sqr ?a)",
        ])
    }

    fn is_allowed_op(&self) -> bool {
        !matches!(self, Trig::Imag | Trig::Cis(_) | Trig::Sqr(_))
    }

    // No eval needed for rule lifting
    fn eval<'a, F>(&'a self, _cvec_len: usize, _get_cvec: F) -> CVec<Self>
    where
        F: FnMut(&'a Id) -> &'a CVec<Self>,
    {
        vec![]
    }

    // No variable initialization needed
    fn initialize_vars(_egraph: &mut EGraph<Self, SynthAnalysis>, _vars: &[String]) {}

    fn to_var(&self) -> Option<Symbol> {
        if let Trig::Var(Variable(sym)) = self {
            Some(*sym)
        } else {
            None
        }
    }

    fn mk_var(sym: Symbol) -> Self {
        Trig::Var(Variable::from(sym.as_str()))
    }

    fn is_constant(&self) -> bool {
        matches!(self, Trig::RealConst(_))
    }

    fn mk_constant(c: Self::Constant, _egraph: &mut EGraph<Self, SynthAnalysis>) -> Self {
        Trig::RealConst(c)
    }

    fn custom_modify(egraph: &mut EGraph<Self, SynthAnalysis>, id: Id) {
        if egraph[id]
            .nodes
            .iter()
            .any(|x| matches!(x, Trig::RealConst(_)))
        {
            return;
        }

        let mut to_add: Option<Trig> = None;
        for n in &egraph[id].nodes {
            match n {
                Trig::Neg(i) => {
                    if let Some(x) = extract_constant(&egraph[*i].nodes) {
                        let r = Real::from((-x).to_string());
                        to_add = Some(Self::mk_constant(r, egraph));
                        break;
                    }
                }
                Trig::Add([i, j]) => {
                    if let Some(x) = extract_constant(&egraph[*i].nodes) {
                        if let Some(y) = extract_constant(&egraph[*j].nodes) {
                            let r = Real::from((x + y).to_string());
                            to_add = Some(Self::mk_constant(r, egraph));
                            break;
                        }
                    }
                }
                Trig::Sub([i, j]) => {
                    if let Some(x) = extract_constant(&egraph[*i].nodes) {
                        if let Some(y) = extract_constant(&egraph[*j].nodes) {
                            let r = Real::from((x - y).to_string());
                            to_add = Some(Self::mk_constant(r, egraph));
                            break;
                        }
                    }
                }
                Trig::Mul([i, j]) => {
                    if let Some(x) = extract_constant(&egraph[*i].nodes) {
                        if let Some(y) = extract_constant(&egraph[*j].nodes) {
                            let r = Real::from((x * y).to_string());
                            to_add = Some(Self::mk_constant(r, egraph));
                            break;
                        }
                    }
                }
                Trig::Div([i, j]) => {
                    if let Some(x) = extract_constant(&egraph[*i].nodes) {
                        if let Some(y) = extract_constant(&egraph[*j].nodes) {
                            if !y.is_zero() {
                                let r = Real::from((x / y).to_string());
                                to_add = Some(Self::mk_constant(r, egraph));
                                break;
                            }
                        }
                    }
                }
                _ => (),
            }
        }

        if let Some(v) = to_add {
            // add (~ v) if v is negative or 0
            if let Trig::RealConst(n) = v {
                if let Ok(x) = n.as_str().parse::<Rational>() {
                    if x.is_negative() || x.is_zero() {
                        let pos_id = egraph.add(Self::mk_constant(
                            Real::from((-x).to_string()),
                            &mut egraph.clone(),
                        ));
                        let neg_id = egraph.add(Trig::Neg(pos_id));
                        egraph.union(neg_id, id);
                    }
                }
            }

            let cnst_id = egraph.add(v);
            egraph.union(cnst_id, id);
        }
    }

    fn validate(_lhs: &Pattern<Self>, _rhs: &Pattern<Self>) -> ValidationResult {
        ValidationResult::Valid
    }
}

impl Trig {
    pub fn run_workload(workload: Workload, prior: Ruleset<Self>, limits: Limits) -> Ruleset<Self> {
        let t = Instant::now();

        let egraph = workload.to_egraph::<Self>();
        let num_prior = prior.len();
        let mut candidates = Ruleset::allow_forbid_actual(egraph, prior.clone(), limits);

        let chosen = candidates.minimize(prior, limits);
        let time = t.elapsed().as_secs_f64();

        println!(
            "Learned {} bidirectional rewrites ({} total rewrites) in {} using {} prior rewrites",
            chosen.bidir_len(),
            chosen.len(),
            time,
            num_prior
        );

        chosen.pretty_print();

        chosen
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ruler::{
        enumo::{Filter, Ruleset, Workload},
        Limits,
    };

    // Extra rules about `cis` and `I` to "fast-forward" rule synthesis
    fn prior_forbidden_rules() -> Ruleset<Trig> {
        Ruleset::new([
            // constant folding for PI
            "(+ PI PI) ==> (* 2 PI)",
            "(* 2 PI) ==> (+ PI PI)",
            // constant folding for cis
            "(cis 0) ==> 1",
            "(cis (/ PI 2)) ==> I",
            "(cis (~ (/ PI 2))) ==> (~ I)",
            "(cis PI) ==> -1",
            // cis identities
            "(cis (+ ?a ?b)) ==> (* (cis ?a) (cis ?b))",
            "(* (cis ?a) (cis ?b)) ==> (cis (+ ?a ?b))",
            "(cis (- ?a ?b)) ==> (* (cis ?a) (cis (~ ?b)))",
            "(* (cis ?a) (cis (~ ?b))) ==> (cis (- ?a ?b))",
            "(cis (~ ?a)) ==> (/ 1 (cis ?a))",
            "(/ 1 (cis ?a)) ==> (cis (~ ?a))",
            "(* (cis ?a) (cis (~ ?a))) ==> 1",
            // constant folding I
            "(/ 1 I) ==> (~ I)",
            "(* I I) ==> -1",
        ])
    }

    fn og_recipe(prior: &Ruleset<Trig>, limits: Limits) -> Ruleset<Trig> {
        let t_ops = Workload::new(["sin", "cos", "tan"]);
        let consts = Workload::new([
            "0", "(/ PI 6)", "(/ PI 4)", "(/ PI 3)", "(/ PI 2)", "PI", "(* PI 2)",
        ]);
        let app = Workload::new(["(op v)"]);
        let trig_constants =
            app.clone()
                .plug("op", &t_ops)
                .plug("v", &consts)
                .filter(Filter::Invert(Box::new(Filter::Contains(
                    "(tan (/ PI 2))".parse().unwrap(),
                ))));

        let simple_terms = app.clone().plug("op", &t_ops).plug(
            "v",
            &Workload::new(["a", "(~ a)", "(+ PI a)", "(- PI a)", "(+ a a)"]),
        );

        let neg_terms = Workload::new(["(~ x)"]).plug("x", &simple_terms);

        let squares = Workload::new(["(sqr x)"])
            .plug("x", &app)
            .plug("op", &t_ops)
            .plug("v", &Workload::new(["a", "b"]));

        let add = Workload::new(["(+ e e)", "(- e e)"]);

        let sum_of_squares = add.plug("e", &squares);

        let mut all = prior.clone();
        let mut new = Ruleset::<Trig>::default();

        let wkld1 = trig_constants;
        println!("Starting 1");
        let rules1 = Trig::run_workload(wkld1.clone(), all.clone(), limits);
        all.extend(rules1.clone());
        new.extend(rules1.clone());
        // assert_eq!(rules1.len(), 22);

        let wkld2 = Workload::Append(vec![wkld1, simple_terms, neg_terms]);
        println!("Starting 2");
        let rules2 = Trig::run_workload(wkld2.clone(), all.clone(), limits);
        all.extend(rules2.clone());
        new.extend(rules2.clone());
        // assert_eq!(rules2.len(), 12);

        let wkld3 = Workload::Append(vec![wkld2.clone(), sum_of_squares.clone()]);
        println!("Starting 3");
        let rules3 = Trig::run_workload(wkld3, all.clone(), limits);
        all.extend(rules3.clone());
        new.extend(rules3.clone());
        // assert_eq!(rules3.len(), 3);

        new
    }

    fn phase2_recipe(prior: &Ruleset<Trig>, limits: Limits) -> Ruleset<Trig> {
        let mut all = prior.clone();
        let mut new = Ruleset::<Trig>::default();

        let non_square_filter = Filter::Invert(Box::new(Filter::Or(vec![
            Filter::Contains("(* (cos ?x) (cos ?x))".parse().unwrap()),
            Filter::Contains("(* (sin ?x) (sin ?x))".parse().unwrap()),
        ])));

        let two_x_filter = Filter::Invert(Box::new(
            Filter::Contains("(+ ?x ?x)".parse().unwrap()),
        ));

        let trivial_trig_filter = Filter::Invert(Box::new(Filter::Or(vec![
            Filter::Contains("(cos (?op ?a ?b))".parse().unwrap()),
            Filter::Contains("(sin (?op ?a ?b))".parse().unwrap()),
        ])));

        let t_ops = Workload::new(["sin", "cos"]);
        let app = Workload::new(["(op v)"]);
        let shift = Workload::new(["x", "(- 1 x)", "(+ 1 x)"]);
        let scale = Workload::new(["x", "(/ x 2)"]);

        let simple = app.clone().plug("op", &t_ops).plug(
            "v",
            &Workload::new(["a", "(- (/ PI 2) a)", "(+ (/ PI 2) a)", "(* 2 a)"]),
        );

        let squares = Workload::new(["(sqr x)"])
            .plug("x", &app)
            .plug("op", &t_ops)
            .plug("v", &Workload::new(["a"]));

        let two_var = app
            .clone()
            .plug("op", &t_ops)
            .plug("v", &Workload::new(["a", "b", "(+ a b)", "(- a b)"]));
        let sum_two_vars = Workload::new(["(+ x y)", "(- x y)"])
            .plug("x", &two_var)
            .plug("y", &two_var);
        let prod_two_vars = Workload::new(["(* x y)"])
            .plug("x", &two_var)
            .plug("y", &two_var)
            .filter(non_square_filter);

        let sum_of_prod = Workload::new(["(+ x y)", "(- x y)"])
            .plug("x", &prod_two_vars)
            .plug("y", &prod_two_vars)
            .filter(two_x_filter)
            .filter(trivial_trig_filter);

        let sum_and_prod = Workload::Append(vec![sum_two_vars.clone(), prod_two_vars.clone()]);
        let shifted = shift.clone().plug("x", &simple);

        // Coangles
        let wkld1 = simple;
        let rules1 = Trig::run_workload(wkld1.clone(), all.clone(), limits);
        all.extend(rules1.clone());
        new.extend(rules1.clone());

        // Power reduction
        let wkld2 = scale
            .clone()
            .plug("x", &shifted.clone().append(squares.clone()));
        let rules2 = Trig::run_workload(wkld2.clone(), all.clone(), limits);
        all.extend(rules2.clone());
        new.extend(rules2.clone());

        // Product-to-sum
        let wkld3 = scale.clone().plug("x", &sum_and_prod);
        let rules3 = Trig::run_workload(wkld3.clone(), all.clone(), limits);
        all.extend(rules3.clone());
        new.extend(rules3.clone());

        // Sums
        let wkld4 = Workload::Append(vec![two_var, sum_of_prod]);
        let rules4 = Trig::run_workload(wkld4.clone(), all.clone(), limits);
        all.extend(rules4.clone());
        new.extend(rules4.clone());

        new
    }

    #[test]
    fn nightly_recipe() {
        let complex: Ruleset<Trig> = Ruleset::from_file("scripts/trig/complex.rules");
        let limits = Limits {
            iter: 3,
            node: 2000000,
        };

        let mut all = complex;
        let mut new = Ruleset::<Trig>::default();

        // Add prior forbidden rules
        all.extend(prior_forbidden_rules());

        // Run original Enumo recipe
        // let rules = og_recipe(&all, limits);
        let rules = Ruleset::new([
            "(cos (/ PI 2)) ==> 0",
            "0 ==> (cos (/ PI 2))",
            "0 ==> (sin (* PI 2))",
            "(sin (* PI 2)) ==> 0",
            "1 ==> (sin (/ PI 2))",
            "(sin (/ PI 2)) ==> 1",
            "0 ==> (tan (* PI 2))",
            "(tan (* PI 2)) ==> 0",
            "1 ==> (cos (* PI 2))",
            "(cos (* PI 2)) ==> 1",
            "(tan 0) ==> 0",
            "0 ==> (tan 0)",
            "0 ==> (sin 0)",
            "(sin 0) ==> 0",
            "1 ==> (cos 0)",
            "(cos 0) ==> 1",
            "0 ==> (sin PI)",
            "(sin PI) ==> 0",
            "-1 ==> (cos PI)",
            "(cos PI) ==> -1",
            "(tan PI) ==> (sin PI)",
            "(sin PI) ==> (tan PI)",
            "(~ (cos ?a)) ==> (cos (- PI ?a))",
            "(cos (- PI ?a)) ==> (~ (cos ?a))",
            "(sin (- PI ?a)) ==> (sin ?a)",
            "(sin ?a) ==> (sin (- PI ?a))",
            "(tan ?a) ==> (tan (+ PI ?a))",
            "(tan (+ PI ?a)) ==> (tan ?a)",
            "(~ (sin ?a)) ==> (sin (~ ?a))",
            "(sin (~ ?a)) ==> (~ (sin ?a))",
            "(tan (~ ?a)) ==> (~ (tan ?a))",
            "(~ (tan ?a)) ==> (tan (~ ?a))",
            "(cos (~ ?a)) ==> (cos ?a)",
            "(cos ?a) ==> (cos (~ ?a))",
            "(+ (sqr (sin ?a)) (sqr (cos ?a))) ==> 1",
            "(- (sqr (cos ?b)) (sqr (cos ?a))) ==> (- (sqr (sin ?a)) (sqr (sin ?b)))",
            "(- (sqr (sin ?b)) (sqr (cos ?a))) ==> (- (sqr (sin ?a)) (sqr (cos ?b)))",
        ]);

        all.extend(rules.clone());
        new.extend(rules);

        // Run 2nd phase
        let rules = phase2_recipe(&all, limits);
        all.extend(rules.clone());
        new.extend(rules);

        // Only new rules should be uploaded!
        new.write_json_rules("trig.json");
    }

    #[test]
    fn simple() {
        let complex: Ruleset<Trig> = Ruleset::from_file("scripts/trig/complex.rules");
        assert_eq!(complex.len(), 57);

        let limits = Limits {
            iter: 3,
            node: 2000000,
        };

        let terms = Workload::new([
            "(sin 0)",
            "(sin (/ PI 6))",
            "(sin (/ PI 4))",
            "(sin (/ PI 3))",
            "(sin (/ PI 2))",
            "(sin PI)",
            "(sin (* PI 2))",
        ]);
        assert_eq!(terms.force().len(), 7);

        let mut all = complex;
        all.extend(prior_forbidden_rules());

        let rules = Trig::run_workload(terms, all, limits);
        assert_eq!(rules.len(), 6);
    }

    #[test]
    fn sandbox() {
        let complex_rules: Ruleset<Trig> = Ruleset::from_file("scripts/trig/complex.rules");
        let prior_rules: Ruleset<Trig> = Ruleset::new([
            "(cos (/ PI 2)) ==> 0",
            "0 ==> (cos (/ PI 2))",
            "0 ==> (sin (* PI 2))",
            "(sin (* PI 2)) ==> 0",
            "1 ==> (sin (/ PI 2))",
            "(sin (/ PI 2)) ==> 1",
            "0 ==> (tan (* PI 2))",
            "(tan (* PI 2)) ==> 0",
            "1 ==> (cos (* PI 2))",
            "(cos (* PI 2)) ==> 1",
            "(tan 0) ==> 0",
            "0 ==> (tan 0)",
            "0 ==> (sin 0)",
            "(sin 0) ==> 0",
            "1 ==> (cos 0)",
            "(cos 0) ==> 1",
            "0 ==> (sin PI)",
            "(sin PI) ==> 0",
            "-1 ==> (cos PI)",
            "(cos PI) ==> -1",
            "(tan PI) ==> (sin PI)",
            "(sin PI) ==> (tan PI)",
            "(~ (cos ?a)) ==> (cos (- PI ?a))",
            "(cos (- PI ?a)) ==> (~ (cos ?a))",
            "(sin (- PI ?a)) ==> (sin ?a)",
            "(sin ?a) ==> (sin (- PI ?a))",
            "(tan ?a) ==> (tan (+ PI ?a))",
            "(tan (+ PI ?a)) ==> (tan ?a)",
            "(~ (sin ?a)) ==> (sin (~ ?a))",
            "(sin (~ ?a)) ==> (~ (sin ?a))",
            "(tan (~ ?a)) ==> (~ (tan ?a))",
            "(~ (tan ?a)) ==> (tan (~ ?a))",
            "(cos (~ ?a)) ==> (cos ?a)",
            "(cos ?a) ==> (cos (~ ?a))",
            "(+ (sqr (sin ?a)) (sqr (cos ?a))) ==> 1",
            "(- (sqr (cos ?b)) (sqr (cos ?a))) ==> (- (sqr (sin ?a)) (sqr (sin ?b)))",
            "(- (sqr (sin ?b)) (sqr (cos ?a))) ==> (- (sqr (sin ?a)) (sqr (cos ?b)))",
            "(cos (- (/ PI 2) ?a)) ==> (sin ?a)",
            "(sin ?a) ==> (cos (- (/ PI 2) ?a))",
            "(/ (- 1 (cos (+ ?a ?a))) 2) ==> (* (sin ?a) (sin ?a))",
            "(* (sin ?a) (sin ?a)) ==> (/ (- 1 (cos (+ ?a ?a))) 2)",
            "(/ (+ 1 (cos (+ ?a ?a))) 2) ==> (* (cos ?a) (cos ?a))",
            "(* (cos ?a) (cos ?a)) ==> (/ (+ 1 (cos (+ ?a ?a))) 2)",
            "(* (sin ?b) (sin ?a)) ==> (/ (- (cos (- ?a ?b)) (cos (+ ?b ?a))) 2)",
            "(/ (- (cos (- ?a ?b)) (cos (+ ?b ?a))) 2) ==> (* (sin ?b) (sin ?a))",
            "(* (cos ?b) (cos ?a)) ==> (/ (+ (cos (+ ?a ?b)) (cos (- ?b ?a))) 2)",
            "(/ (+ (cos (+ ?a ?b)) (cos (- ?b ?a))) 2) ==> (* (cos ?b) (cos ?a))",
        ]);

        let limits = Limits {
            iter: 3,
            node: 2000000,
        };

        let mut rules = prior_rules.clone();
        rules.extend(complex_rules);
        rules.extend(prior_forbidden_rules());

        // Layer
        let wkld = Workload::new([
            "(cos (+ x y))",
            "(- (* (cos x) (cos y)) (* (sin x) (sin y)))"
        ]);

        let new_rules = Trig::run_workload(wkld, rules.clone(), limits);
        rules.extend(new_rules);
    }
}
