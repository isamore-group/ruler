use egg::*;
use rand::SeedableRng;
use ruler::*;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut param = SynthParam {
        rng: SeedableRng::seed_from_u64(5),
        n_iter: 2,
        n_samples: 25,
        variables: vec!["x".into(), "y".into(), "z".into()],
        consts: vec![-1, 0, 1],
    };

    let eqs = param.run();
    let rules = eqs.iter().flat_map(|eq| &eq.rewrites);

    println!("Entering simplification loop...");
    let stdin = io::stdin();
    loop {
        print!("Input expression: ");
        io::stdout().flush()?;
        let mut expr_str = String::new();
        stdin.read_line(&mut expr_str)?;

        let runner: Runner<SimpleMath, SynthAnalysis, ()> = Runner::default()
            .with_expr(&expr_str.parse().unwrap())
            .run(rules.clone());

        let mut ext = Extractor::new(&runner.egraph, AstSize);
        let (_, simp_expr) = ext.find_best(runner.roots[0]);
        println!("Simplified result: {}", simp_expr);
        println!();
    }
}
