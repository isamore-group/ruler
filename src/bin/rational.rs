use ruler::*;
use egg::*;

use rand_pcg::Pcg64;
use num::bigint::{BigInt, RandBigInt, ToBigInt};
use num::{Zero, rational::Ratio, Signed, ToPrimitive};

pub type Constant = Ratio<BigInt>;

define_language! {
    pub enum Math {
        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "/" = Div([Id; 2]),
        "pow" = Pow([Id; 2]),
        "fabs" = Abs(Id),
        "recip" = Reciprocal(Id),
        "~" = Neg(Id),
        Num(Constant),
        Var(egg::Symbol),
    }
}

fn mk_constant(n: &BigInt, d: &BigInt) -> Option<Constant> {
    if d.is_zero() {
        None
    } else {
        Some(Ratio::new(n.clone(), d.clone()))
    }
}

impl SynthLanguage for Math {
    type Constant = Constant;

    fn eval<'a, F>(&'a self, cvec_len: usize, mut v: F) -> CVec<Self>
    where
        F: FnMut(&'a Id) -> &'a CVec<Self> {
        match self {
            Math::Neg(a) => map!(v, a => Some(-a)),
            Math::Add([a, b]) => map!(v, a, b => Some(a + b)),
            Math::Sub([a, b]) => map!(v, a, b => Some(a - b)),
            Math::Mul([a, b]) => map!(v, a, b => Some(a * b)),
            Math::Num(n) => vec![Some(n.clone()); cvec_len],
            Math::Var(_) => vec![],
            Math::Div([a, b]) => map!(v, a, b => {
                if b.is_zero() {
                    None
                } else{
                    Some(a / b)
                }
            }),
            Math::Abs(a) => map!(v, a => Some(a.abs())),
            Math::Pow([a, b]) => map!(v, a, b => {
                b.to_i32().map(|b| a.pow(b))
            }),
            Math::Reciprocal(a) => map!(v, a => {
                if a.is_zero() {
                    None
                } else {
                    Some(a.recip())
                }
            }),
        }
    }

    fn to_var(&self) -> Option<Symbol> {
        if let Math::Var(sym) = self {
            Some(*sym)
        } else {
            None
        }
    }

    fn mk_var(sym: Symbol) -> Self {
        Math::Var(sym)
    }

    fn to_constant(&self) -> Option<&Self::Constant> {
        if let Math::Num(n) = self {
            Some(n)
        } else {
            None
        }
    }

    fn mk_constant(c: Self::Constant) -> Self {
        Math::Num(c)
    }

    fn init_synth(synth: &mut Synthesizer<Self>) {
        let params = &synth.params;
        let mut egraph = EGraph::new(SynthAnalysis {
            // cvec_len: params.n_samples + params.constants.len(),
            cvec_len: params.n_samples + params.constants.len().pow(params.variables as u32),
        });
        let rng = &mut synth.rng;
        for i in 0..params.variables {
            let var = Symbol::from(letter(i));
            let id = egraph.add(Math::Var(var));

            egraph[id].data.cvec = (0..params.n_samples)
                .map(|_| mk_constant(&rng.gen_bigint(32), &gen_denom(rng, 32)))
                .chain(chain_consts(
                    params.constants.clone(),
                    params.variables as u32,
                    i as u32,
                ))
                .collect();

            // let mut cvec: Vec<Option<Constant>> = (0..params.n_samples)
            //     .map(|_| mk_constant(&rng.gen_bigint(32), &gen_denom(&rng, 32)))
            //     .collect();
            // for c in &params.constants {
            //     cvec.push(mk_constant(c.numer(), c.denom()));
            // }
            // egraph[id].data.cvec = cvec.clone();
        }

        for n in &params.constants {
            egraph.add(Math::Num(n.clone()));
        }

        synth.egraph = egraph;
    }

    fn make_layer(synth: &Synthesizer<Self>) -> Vec<Self> {
        let mut to_add = vec![];
        for i in synth.ids() {
            for j in synth.ids() {
                if synth.egraph[i].data.exact && synth.egraph[j].data.exact {
                    continue;
                }
                to_add.push(Math::Add([i, j]));
                //to_add.push(Math::Sub([i, j]));
                to_add.push(Math::Mul([i, j]));
                to_add.push(Math::Div([i, j]));
                // to_add.push(Math::Pow([i, j]));
            }
            if synth.egraph[i].data.exact {
                continue;
            }
            // to_add.push(Math::Abs(i));
            // to_add.push(Math::Reciprocal(i));
            // to_add.push(Math::Neg(i));
        }

        log::info!("Made a layer of {} enodes", to_add.len());
        to_add
    }

    fn is_valid(_lhs: &Pattern<Self>, _rhs: &Pattern<Self>) -> bool {
        true
    }
}

fn chain_consts(constants: Vec<Constant>, nvars: u32, i: u32) -> Vec<Option<Constant>> {
    let mut res = vec![];
    let mut consts = vec![];
    for c in constants {
        consts.push(mk_constant(c.numer(), c.denom()));
    }
    let nc = consts.len();
    let nrows = nc.pow(nvars as u32);
    while res.len() < nrows {
        for c in &consts {
            for _ in 0..nc.pow(i) {
                res.push(c.clone())
            }
        }
    }
    res
}

// randomly sample denoms so that they are not 0
// Ratio::new will panic if the denom is 0
pub fn gen_denom(rng: &mut Pcg64, bits: u64) -> BigInt {
    let mut res: BigInt;
    loop {
        res = rng.gen_bigint(bits);
        if res != 0.to_bigint().unwrap() {
            break;
        }
    }
    res
}

fn main() {
    let _ = env_logger::builder().try_init();
    let syn = Synthesizer::<Math>::new(SynthParams {
        seed: 5,
        n_samples: 10,
        constants: vec![
            Ratio::new(0.to_bigint().unwrap(), 1.to_bigint().unwrap()),
            Ratio::new(1.to_bigint().unwrap(), 1.to_bigint().unwrap()),
            Ratio::new(-1.to_bigint().unwrap(), 1.to_bigint().unwrap())
        ],
        variables: 2,
        iters: 2,
        rules_to_take: 1,
        chunk_size: usize::MAX,
        minimize: true,
        outfile: "minimize.json".to_string()
    });
    let outfile = &syn.params.outfile.clone();
    let report = syn.run();

    let file = std::fs::File::create(outfile)
        .unwrap_or_else(|_| panic!("Failed to open '{}'", outfile));
    serde_json::to_writer_pretty(file, &report).expect("failed to write json");
}
