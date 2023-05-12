use ruler::{
    enumo::{Filter, Ruleset, Workload},
    recipe_utils::{base_lang, iter_metric, recursive_rules, run_workload, Lang},
};

ruler::impl_bv!(4);

pub fn bv4_fancy_rules() -> Ruleset<Bv> {
    let mut rules: Ruleset<Bv> = Ruleset::default();
    let lang = Lang::new(
        &["0", "1"],
        &["a", "b", "c"],
        &["~", "-"],
        &["&", "|", "*", "--", "+", "<<", ">>"],
        &[],
    );
    rules.extend(recursive_rules(
        enumo::Metric::Atoms,
        5,
        lang.clone(),
        Ruleset::default(),
    ));

    let a6_canon = iter_metric(base_lang(2), "EXPR", enumo::Metric::Atoms, 6)
        .plug("VAR", &Workload::new(lang.vars))
        .plug("VAL", &Workload::empty())
        .plug("OP1", &Workload::new(lang.uops))
        .plug("OP2", &Workload::new(lang.bops))
        .filter(Filter::Canon(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]));

    rules.extend(run_workload(
        a6_canon,
        rules.clone(),
        Limits::rulefinding(),
        true,
    ));
    rules
}
