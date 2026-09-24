use rand::{Rng, RngExt, SeedableRng};

pub mod matrices;

#[derive(Clone, Copy, Debug)]
pub struct GameSettings {
    pub n_rounds: usize,
    pub payout: [f64; 4],  // index msb=mine, lsb=theirs
    pub miss_rate: f64,
}

impl GameSettings {
    pub fn payout_at(&self, mine: bool, theirs: bool) -> f64 {
        let i = if mine { 2 } else { 0 } | if theirs { 1 } else { 0 };
        self.payout[i]
    }
    pub fn run_game(&self, p1: &mut impl Strategy, p2: &mut impl Strategy) -> (f64, f64) {
        if self.n_rounds == 0 {
            (0.0, 0.0)
        } else {
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(rand::rng().next_u64());
            let mut o1 = p1.decide(self, None) ^ rng.random_bool(self.miss_rate);
            let mut o2 = p2.decide(self, None) ^ rng.random_bool(self.miss_rate);
            let mut y1 = self.payout_at(o1, o2);
            let mut y2 = self.payout_at(o2, o1);
            for _ in 1..self.n_rounds {
                let x1 = p1.decide(self, Some((o1, o2))) ^ rng.random_bool(self.miss_rate);
                let x2 = p2.decide(self, Some((o2, o1))) ^ rng.random_bool(self.miss_rate);
                (o1, o2) = (x1, x2);
                y1 += self.payout_at(o1, o2);
                y2 += self.payout_at(o2, o1);
            }
            (y1, y2)
        }
    }
}

pub trait Strategy {
    fn label(&self) -> String;
    fn decide(&mut self, game: &GameSettings, update: Option<(bool, bool)>) -> bool;
}

impl<T: ?Sized + Strategy> Strategy for Box<T> {
    fn label(&self) -> String { self.as_ref().label() }
    fn decide(&mut self, game: &GameSettings, update: Option<(bool, bool)>) -> bool {
        self.as_mut().decide(game, update)
    }
}

pub struct Periodic(usize, Box<[bool]>);
impl Periodic {
    pub fn new(sequence: impl Into<Box<[bool]>>) -> Self { Self(0, sequence.into()) }
    pub fn mk_generous() -> Self { Self::new([true]) }
    pub fn mk_greedy() -> Self { Self::new([false]) }
    pub fn mk_alternate(start: bool, half_period: usize) -> Self {
        let mut buf = vec![start; half_period];
        buf.extend((0..half_period).map(|_| !start));
        Self::new(buf)
    }
}
impl Strategy for Periodic {
    fn label(&self) -> String {
        let inner = self.1.iter().map(|&p| if p { '1' } else { '0' }).collect::<String>();
        format!("Periodic{{{}}}", inner)
    }
    fn decide(&mut self, _: &GameSettings, _: Option<(bool, bool)>) -> bool {
        let i = self.0;
        let n = self.1.len();
        self.0 = (i + 1) % n;
        self.1[i]
    }
}

pub struct ByLast {
    label: String,
    opening: bool,
    f: Box<dyn FnMut(bool, bool) -> bool>,
}
impl ByLast {
    pub fn new(label: String, opening: bool, f: impl 'static + Sized + FnMut(bool, bool) -> bool) -> Self {
        Self { label, opening, f: Box::new(f) }
    }
    pub fn mk_grudger(mut threshold: usize) -> Self {
        Self::new(format!("Grudger({})", threshold), true, move |_, theirs| {
            if !theirs && threshold > 0 { threshold -= 1; }
            threshold != 0
        })
    }
    pub fn mk_thankful(mut threshold: usize) -> Self {
        Self::new(format!("Thankful({})", threshold), false, move |_, theirs| {
            if theirs && threshold > 0 { threshold -= 1; }
            threshold == 0
        })
    }
    pub fn mk_copycat(mut nice: bool, invert: bool, streak_to_copy: usize, streak_to_nice: usize) -> Self {
        let mut label = String::new();
        if nice { label.push_str("nice,"); }
        if invert { label.push_str("invert,"); }
        let label = format!("Copycat({}{},{})", label, streak_to_copy, streak_to_nice);
        let mut streak = 0;
        Self::new(label, !invert, move |_, theirs| {
            if nice != theirs {
                streak += 1;
            } else {
                streak = 0;
            }
            if nice && streak >= streak_to_copy {
                nice = false; streak = 0;
            } else if !nice && streak >= streak_to_nice {
                nice = true; streak = 0;
            }
            invert ^ (nice || theirs)
        })
    }
    pub fn mk_pavlov() -> Self {
        Self::new("Pavlov".to_owned(), true, |mine, theirs| mine == theirs)
    }
}
impl Strategy for ByLast {
    fn label(&self) -> String { self.label.to_owned() }
    fn decide(&mut self, _: &GameSettings, update: Option<(bool, bool)>) -> bool {
        match update {
            Some((mine, theirs)) => (self.f)(mine, theirs),
            None => self.opening,
        }
    }
}

pub struct Detective {
    n_rounds: usize,
    exploit: bool,
}
impl Detective {
    pub fn new() -> Self { Self { n_rounds: 0, exploit: true } }
}
impl Strategy for Detective {
    fn label(&self) -> String { "Detective".to_owned() }
    fn decide(&mut self, _: &GameSettings, update: Option<(bool, bool)>) -> bool {
        match update {
            None => {
                assert_eq!(self.n_rounds, 0);
                self.n_rounds += 1;
                true
            }
            Some((_, theirs)) if self.n_rounds < 4 => {
                if self.n_rounds == 3 {
                    self.exploit &= theirs;
                }
                self.n_rounds += 1;
                self.n_rounds != 2
            }
            Some((_, theirs)) => !self.exploit && theirs,
        }
    }
}

pub struct CoinToss {
    heads: f64,
    rng: rand_chacha::ChaCha8Rng,
}
impl CoinToss {
    pub fn new(heads: f64, seed: u64) -> Self {
        Self { heads, rng: rand_chacha::ChaCha8Rng::seed_from_u64(seed) }
    }
}
impl Strategy for CoinToss {
    fn label(&self) -> String { format!("CoinToss({})", self.heads) }
    fn decide(&mut self, _: &GameSettings, _: Option<(bool, bool)>) -> bool {
        self.rng.random_bool(self.heads)
    }
}

pub struct Betrayer {
    percentage: f64,
    count: usize,
}
impl Betrayer {
    pub fn new(percentage: f64) -> Self { Self { percentage, count: 0 } }
}
impl Strategy for Betrayer {
    fn label(&self) -> String { format!("Betrayer({})", self.percentage) }
    fn decide(&mut self, g: &GameSettings, update: Option<(bool, bool)>) -> bool {
        match update {
            None => true,
            Some((_, theirs)) => {
                self.count += 1;
                (self.count as f64) < (g.n_rounds as f64) * self.percentage && theirs
            }
        }
    }
}

pub struct Ingroup {
    label: String,
    tolerance: usize,
    displays: Box<[bool]>,
    expects: Box<[bool]>,
    policies: [Box<dyn Strategy>; 2],
    count: usize,
    match_distance: usize,
}
impl Ingroup {
    pub fn new(
        label: String,
        tolerance: usize,
        displays: impl 'static + Sized + Into<Box<[bool]>>,
        expects: impl 'static + Sized + Into<Box<[bool]>>,
        for_foes: impl 'static + Sized + Strategy,
        for_friends: impl 'static + Sized + Strategy,
    ) -> Self {
        let displays = displays.into();
        let expects = expects.into();
        let policies: [Box<dyn Strategy>; _] =
            [Box::new(for_foes), Box::new(for_friends)];
        Self { label, tolerance, displays, expects, policies, count: 0, match_distance: 0 }
    }
    fn bits_with_labels(
        pattern: impl 'static + Sized + Into<Box<[bool]>>,
    ) -> (Box<[bool]>, String) {
        let bits = pattern.into();
        let label = bits.iter().map(|p| if *p {'1'} else {'0'}).collect::<String>();
        (bits, label)
    }
    pub fn mk_cult_leader(
        tolerance: usize,
        center: impl 'static + Sized + Into<Box<[bool]>>,
        periph: impl 'static + Sized + Into<Box<[bool]>>,
    ) -> Self {
        let (center, clabel) = Self::bits_with_labels(center);
        let (periph, plabel) = Self::bits_with_labels(periph);
        let label = format!("CultLeader({}){{{},{}}}", tolerance, clabel, plabel);
        Self::new(
            label, tolerance, center, periph,
            ByLast::mk_copycat(true, false, 1, 1),
            Periodic::mk_greedy(),
        )
    }
    pub fn mk_cult_member(
        tolerance: usize,
        center: impl 'static + Sized + Into<Box<[bool]>>,
        periph: impl 'static + Sized + Into<Box<[bool]>>,
    ) -> Self {
        let (center, clabel) = Self::bits_with_labels(center);
        let (periph, plabel) = Self::bits_with_labels(periph);
        let label = format!("CultMember({}){{{},{}}}", tolerance, clabel, plabel);
        Self::new(
            label, tolerance, periph, center,
            Periodic::mk_greedy(),
            Periodic::mk_generous(),
        )
    }
    pub fn mk_green_beard(
        tolerance: usize,
        key: impl 'static + Sized + Into<Box<[bool]>>,
    ) -> Self {
        let (key, bitstr) = Self::bits_with_labels(key);
        let label = format!("GreenBeard({}){{{}}}", tolerance, bitstr);
        Self::new(
            label, tolerance, key.clone(), key,
            Periodic::mk_greedy(),
            Periodic::mk_generous(),
        )
    }
    pub fn mk_fake_beard(
        tolerance: usize,
        key: impl 'static + Sized + Into<Box<[bool]>>,
    ) -> Self {
        let (key, bitstr) = Self::bits_with_labels(key);
        let label = format!("FakeBeard({}){{{}}}", tolerance, bitstr);
        Self::new(
            label, tolerance, key.clone(), key,
            ByLast::mk_copycat(true, false, 1, 1),
            Periodic::mk_greedy(),
        )
    }
}
impl Strategy for Ingroup {
    fn label(&self) -> String { self.label.clone() }
    fn decide(&mut self, g: &GameSettings, update: Option<(bool, bool)>) -> bool {
        let ch = [
            self.policies[0].decide(g, update.clone()),
            self.policies[1].decide(g, update.clone()),
        ];
        match update {
            None => {
                assert_eq!(self.count, 0); self.count += 1;
            }
            Some((_, theirs)) => {
                assert_ne!(self.count, 0); self.count += 1;
                let i = self.count - 2;
                if i < self.expects.len() {
                    self.match_distance += if theirs != self.expects[i] {1} else {0};
                    // if i + 1 == self.expects.len() {
                    //     println!("{} locked into friend={}", self.label, self.match_distance <= self.tolerance);
                    // }
                }
            }
        }
        let i = self.count - 1;
        if i < self.displays.len() {
            self.displays[i]
        } else {
            ch[if self.match_distance <= self.tolerance {1} else {0}]
        }
    }
}

fn bernouli_mean_var(total: f64, success: f64) -> (f64, f64) {
    let μ = success / total;
    let var = μ * (1.0 - μ) / total;
    (μ, var)
}

pub struct Businessman {
    z: f64, // required confidence sigmas
    b: f64, // detect imbalanced samples by thresholding real stdev / ideal stdev
    decay: f64, // discount old impressions
    opening: Box<[bool]>,

    i: usize, // opening index
    n: [f64; 2], // total
    m: [f64; 2], // coincidental
    my_last: bool, // coincidence check happens across turns, so this is necessary
}
impl Businessman {
    pub fn new(
        z: f64, b: f64, decay: f64,
        opening: impl 'static + Sized + Into<Box<[bool]>>,
    ) -> Self {
        assert!(0.0 <= decay && decay <= 1.0);
        let opening = opening.into();
        assert!(opening.len() >= 2);
        Self { z, b, decay, opening, i: 0, n: [0., 0.], m: [0., 0.], my_last: false }
    }
    fn approximate_linear(payout: &[f64; 4]) -> (f64, f64, f64) {
        let &[d, c, b, a] = payout;
        // we assume the payout is linear, in the form of f(u,v) = w + su + tv
        // we estimate w, s, t
        //     w + s + t = a
        //     w + s     = b
        //     w     + t = c
        //     w         = d
        // we recognize that this is an overdetermined set of linear equations in the form
        //   A x == b
        // and hence we proceed by solving
        //   A⊤ A x = A⊤ b
        let t = 0.5 * (a - b + c - d);
        let s = 0.5 * (a + b - c - d);
        let w = 0.25 * (-a + b + c + 3.0*d);
        (w, s, t)
    }
}
impl Strategy for Businessman {
    fn label(&self) -> String {
        let opening = self.opening.iter().map(|&p| if p {'1'} else {'0'}).collect::<String>();
        format!("Businessman({},{},{}){{{}}}", self.z, self.b, self.decay, opening)
    }
    fn decide(&mut self, g: &GameSettings, update: Option<(bool, bool)>) -> bool {
        match update {
            None => {
                assert_eq!(self.i, 0);
                self.i += 1;
                self.opening[0]
            }
            Some((mine, theirs)) => {
                assert_ne!(self.i, 0);
                self.n[0] *= self.decay; self.n[1] *= self.decay;
                self.m[0] *= self.decay; self.m[1] *= self.decay;
                if self.i >= 2 {
                    let j = if self.my_last {1} else {0};
                    self.n[j] += 1.0;
                    if self.my_last == theirs { self.m[j] += 1.0; }
                }
                self.my_last = mine;
                let old_i = self.i;
                self.i += 1;
                if old_i < self.opening.len() {
                    return self.opening[old_i]
                }
                let (μ0, var0) = bernouli_mean_var(self.n[0], self.m[0]);
                let (μ1, var1) = bernouli_mean_var(self.n[1], self.m[1]);
                let r = μ0 + μ1 - 1.0;
                let sd_r = (var0 + var1).sqrt();
                let sd_r_ideal =
                    ((μ0 * (1.0 - μ0) + μ1 * (1.0 - μ1)) * 2.0 / (self.n[0] + self.n[1])).sqrt();
                let (_w, s, t) = Self::approximate_linear(&g.payout);
                let (r_lo, r_hi) = (r - sd_r * self.z, r + sd_r * self.z);
                if s + t * r_lo > 0.0 {
                    true
                } else if s + t * r_hi >= 0.0 {
                    if sd_r / sd_r_ideal > self.b {
                        self.n[0] >= self.n[1]
                    } else {
                        theirs
                    }
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
}
