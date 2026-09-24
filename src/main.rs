use rand::{Rng, SeedableRng, distr::{Distribution, weighted::WeightedIndex}};
use rayon::prelude::*;

use quicktrust::*;
use quicktrust::matrices::*;

fn main() {

    // candidate strategies; they don't replicate in instance, but in weight.
    // duplicate strategies are unnecessary since they can just be reweighed;
    // variant strategies are discounted.
    let strategies: [(f32, Box<dyn Send + Sync + Fn() -> Box<dyn Strategy>>); _] = [
        // (1.000, Box::new(|| Box::new(Periodic::mk_nice()))),
        // (1.000, Box::new(|| Box::new(Periodic::mk_evil()))),
        // (1.000, Box::new(|| Box::new(Periodic::mk_tidal(true, 1)))),
        // (1.000, Box::new(|| Box::new(Periodic::mk_tidal(true, 2)))),
        // (1.000, Box::new(|| Box::new(Periodic::mk_tidal(true, 5)))),
        // (1.000, Box::new(|| Box::new(Periodic::mk_tidal(false, 1)))),
        // (1.000, Box::new(|| Box::new(ByLast::mk_grudger(1)))),
        // (1.000, Box::new(|| Box::new(ByLast::mk_grudger(2)))),
        // (1.000, Box::new(|| Box::new(ByLast::mk_thankful(1)))),
        // (1.000, Box::new(|| Box::new(ByLast::mk_thankful(2)))),
        // (1.000, Box::new(|| Box::new(ByLast::mk_copycat(true, false, 1, 1)))),
        // (0.333, Box::new(|| Box::new(ByLast::mk_copycat(true, false, 2, usize::MAX)))),
        // (0.333, Box::new(|| Box::new(ByLast::mk_copycat(true, false, 2, 2)))),
        // (0.333, Box::new(|| Box::new(ByLast::mk_copycat(true, false, 2, 1)))),
        // (1.000, Box::new(|| Box::new(ByLast::mk_copycat(true, true, 1, 1)))),
        // (1.000, Box::new(|| Box::new(ByLast::mk_pavlov()))),
        // (0.333, Box::new(|| Box::new(CoinToss::new(0.25, rand::rng().next_u64())))),
        // (0.333, Box::new(|| Box::new(CoinToss::new(0.50, rand::rng().next_u64())))),
        // (0.333, Box::new(|| Box::new(CoinToss::new(0.75, rand::rng().next_u64())))),
        // (0.250, Box::new(|| Box::new(Betrayer::new(0.20)))),
        // (0.250, Box::new(|| Box::new(Betrayer::new(0.40)))),
        // (0.250, Box::new(|| Box::new(Betrayer::new(0.60)))),
        // (0.250, Box::new(|| Box::new(Betrayer::new(0.80)))),
        // (1.000, Box::new(|| Box::new(Ingroup::mk_cult_leader(
        //     1, [true, false, false, true, true, false, true, false], [false, true, true, false, false, true, false, true])))),
        // (5.000, Box::new(|| Box::new(Ingroup::mk_cult_member(
        //     1, [true, false, false, true, true, false, true, false], [false, true, true, false, false, true, false, true])))),
        // (1.200, Box::new(|| Box::new(Ingroup::mk_green_beard(1, [false, true, false, true, true, true, false, false])))),
        // (1.000, Box::new(|| Box::new(Detective::new()))),
        // (0.333, Box::new(|| Box::new(Businessman::new(1.5, 1.25, 0.98, [true, true, false, true, true, false, true, true])))),
        // (0.333, Box::new(|| Box::new(Businessman::new(1.8, 1.35, 0.98, [true, true, false, true, true, false, true, true])))),
        // (0.333, Box::new(|| Box::new(Businessman::new(2.1, 1.45, 0.98, [true, true, false, true, true, false, true, true])))),

        (1.000, Box::new(|| Box::new(Periodic::mk_nice()))),
        (1.000, Box::new(|| Box::new(Periodic::mk_evil()))),
        (1.000, Box::new(|| Box::new(Periodic::mk_tidal(false, 1)))),
        (1.000, Box::new(|| Box::new(ByLast::mk_grudger(1)))),
        (1.000, Box::new(|| Box::new(ByLast::mk_copycat(true, false, 1, 1)))),
        (1.000, Box::new(|| Box::new(ByLast::mk_copycat(true, false, 2, 1)))),
        (1.000, Box::new(|| Box::new(ByLast::mk_pavlov()))),
        (1.000, Box::new(|| Box::new(Detective::new()))),
        (1.000, Box::new(|| Box::new(Betrayer::new(0.80)))),
        (1.000, Box::new(|| Box::new(Ingroup::mk_green_beard(1, [false, true, false, true, true, true, false, false])))),
        (1.000, Box::new(|| Box::new(Ingroup::mk_fake_beard(1, [false, true, false, true, true, true, false, false])))),
        (1.000, Box::new(|| Box::new(Businessman::new(2.1, 1.45, 0.98, [true, true, false, true, true, false, true, true])))),
    ];
    let n = strategies.len();

    // game settings (one game)
    let game = GameSettings {
        n_rounds: 1_000,
        payout: [0.0, 3.0, -1.0, 2.0],
        miss_rate: 0.05,
    };
    // game settings (one pair of contestants; they're reset between games)
    let n_games = 10_000;
    // simulation settings
    let n_generations = 200;
    let report_every = 50;
    let virtual_population = 10_000; // 0: disable self-play discount
    let multinomial_sampling = true;
    assert!(virtual_population > 0 || !multinomial_sampling);

    println!(
        "per-turn average gain, {} games, {} rounds;\npayout {:?} with miss rate {};",
        n_games, game.n_rounds, game.payout, game.miss_rate,
    );
    println!("even-weight (EW) gain, EW contrib, EW contrastive, diagonal, strategy identifier:");

    // print basic information + create contribution matrix
    let mut contrib = mkmat([n, n], [n, 1], &vec![0.0; strategies.len() * strategies.len()]);
    let mut names = vec![];
    let mut weights = vec![];
    for (i, (w, p1)) in strategies.iter().enumerate() {
        let mut gain = 0.0;
        let mut trib = 0.0;
        let mut diag = 0.0;
        for (j, (_, p2)) in strategies.iter().enumerate() {
            let (score1, score2) = (0..n_games).into_par_iter()
                .map(|_| game.run_game(&mut p1(), &mut p2()))
                .reduce(|| (0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));
            gain += score1;
            trib += score2;
            if i == j { diag = score1; }
            contrib[[i, j]] += score1 as f32;
            contrib[[j, i]] += score2 as f32;
        }
        let name = p1().label();
        println!(
            "{:>8.4}{:>8.4}{:>8.4}{:>8.4}  {}",
            gain / n_games as f64 / game.n_rounds as f64 / n as f64,
            trib / n_games as f64 / game.n_rounds as f64 / n as f64,
            (gain - trib) / n_games as f64 / game.n_rounds as f64 / n as f64,
            diag / n_games as f64 / game.n_rounds as f64,
            name,
        );
        names.push(name);
        weights.push(w);
    }
    println!();
    // make contribution matrix nicer by downscaling so it has mean 1/N where N is candidate count
    // (scale doesn't matter, but float precision kinda does)
    matscl(0.5 / n_games as f32 / game.n_rounds as f32, (&mut contrib, false));
    // strip mutability
    let contrib = contrib;
    let names = names;
    let ini_weights_sum = weights.iter().cloned().sum::<f32>();
    let ini_weights = weights.into_iter().map(|w| w / ini_weights_sum).collect::<Vec<_>>();
    // preprocess the diagonal into a discount vector
    let selfplay_discount =
        if virtual_population > 0 {
            mkmat(
                [n, 1], [1, 1],
                &(0..n).map(|i| contrib[[i, i]] / virtual_population as f32).collect::<Vec<_>>(),
            )
        } else {
            mkmat([n, 1], [1, 1], &vec![0.0; n])
        };

    println!("contribution matrix dump");
    for i in 0..n {
        let rowstr = (0..n).map(|j| format!(" {:5.2}", contrib[[i, j]])).collect::<String>();
        println!("  {:40}{}", names[i], rowstr);
    }
    println!();

    // initialize the candidate weights (represents population)
    let mut census = mkmat([n, 1], [1, 1], &ini_weights);
    // this is where we store the candidate weights for the next round
    let mut census_new: Box<[f32]> = vec![0.0; n].into();
    // this is where we store fitness
    let mut fitness = mkmat([n, 1], [1, 1], &vec![0.0; n]);
    // just a slot from which to retrieve scalars resulting from vector dot products
    let mut scalar = mkmat([1, 1], [1, 1], &[0.0]);
    // pRNG used for discrete virtual population sampling
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(rand::rng().next_u64());
    // this is how we display the results
    fn report_census(population: usize, names: &[String], cen: &BlockedMatrix) {
        let population = if population == 0 {
            println!("(per 1000)");
            1000
        } else {
            population
        };
        let mut tally = names.iter().enumerate()
            .map(|(i, name)| (name, (cen[[i, 0]] * population as f32).round() as u32))
            .filter(|(_, x)| *x > 0)
            .collect::<Vec<_>>();
        tally.sort_by_key(|(_, x)| u32::MAX - *x);
        for (name, count) in tally.into_iter() {
            println!("{:>6}  {}", count, name);
        }
    }
    // vvv  Actual Evolution Simulation Begins Here  vvv
    for i_gen in 0..n_generations {
        if i_gen % report_every == 0 {
            println!(
                "generation {}/{}, N={} (0 disables selfplay discount).",
                i_gen, n_generations, virtual_population,
            );
            if multinomial_sampling {
                println!("multinomial sampling is ON. for large N, this can be slow.");
            } else {
                println!("multinomial sampling is off.");
            }
            report_census(virtual_population, &names, &census);
            println!();
        }
        matmul((&contrib, false), (&census, false), (&mut fitness, false));
        matadd((&selfplay_discount, false), -1.0, (&mut fitness, false));
        matmul((&census, true), (&fitness, false), (&mut scalar, false));
        let fit_sum = scalar[[0, 0]];
        for i in 0..n { census_new[i] = (census[[i, 0]] * (fitness[[i, 0]] / fit_sum)).max(0.0); }
        if let Some(dist) = multinomial_sampling
            .then(|| WeightedIndex::new(&census_new).ok()).flatten()
        {
            matscl(0.0, (&mut census, false));
            for i in (0..virtual_population).map(|_| dist.sample(&mut rng)) {
                census[[i, 0]] += 1.0;
            }
            matscl(1.0 / virtual_population as f32, (&mut census, false));
        } else {
            let census_new_total = census_new.iter().sum::<f32>();
            for i in 0..n { census[[i, 0]] = census_new[i] / census_new_total; }
        }
    }
    // don't forget to report the census one last time
    println!(
        "generation {}/{}, N={} (0 disables selfplay discount).",
        n_generations, n_generations, virtual_population,
    );
    if multinomial_sampling {
        println!("multinomial sampling is ON. for large N, this can be slow.");
    } else {
        println!("multinomial sampling is off.");
    }
    report_census(virtual_population, &names, &census);

}
