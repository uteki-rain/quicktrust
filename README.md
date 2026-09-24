# quicktrust: Prolonged Prisoner Dilemma Simulations, Fast.

[Nicky Case's _The Evolution of Trust_](https://ncase.me/trust) is a parable, an interpretation of the simulation, instead of the full story. I have confidence to say this because through what little experiment I have done, I have already observed much more intricate (and sometimes sinister) behaviors. This repository has two goals:

1. The Game of Trust, sped up, at scale, with the most interesting/competent strategies – and an accompanying evolution simulation capable of being evaluated cheaply for thousands of generations.
1. Developing and improving a set of custom strategies; currently this comprises primarily of the Businessman alone.

## The Game

- Two agents play each other **simultaneously,** for **multiple rounds,** with **no communication**, but **with the full history** of the current game.
- At each turn, each agent can choose to **Collaborate (C) or Defect (D).**
- To simulate error, there is an independent probability $\epsilon$ at each turn for each agent's action to be inverted.

| My Action              | Their Action           | My Gain | Their Gain |
| ---------------------- | ---------------------- | ------- | ---------- |
| **<u>C</u>**ollaborate | **<u>C</u>**ollaborate | $a$     | $a$        |
| **<u>C</u>**ollaborate | **<u>D</u>**efect      | $b$     | $c$        |
| **<u>D</u>**efect      | **<u>C</u>**ollaborate | $c$     | $b$        |
| **<u>D</u>**efect      | **<u>D</u>**efect      | $d$     | $d$        |

- In the classic setup, $a=2,b=-1,c=3,d=0$.
- In general, we need $b<d<a<c$ to constitute a Prisoner's Dilemma.
- **CAVEAT:** In the classic game, the agents are NOT aware of the global landscape of opponents and do NOT have history of any game before the current one.
- **IMPLEMENTATION CAVEAT:** In my implementation, agents can see their own actions *after* the error has been applied, so they can realize their mistakes; in the future, this might become a togglable configuration.
- **NOTE:** Because of the multi-round nature of each game, I present an argument in later sections that this is not a real Prisoner's Dilemma – hence why I'm calling it instead "The Game of Trust".

## The Evolution

- We maintain a pool where the population of strategy $s$ at generation $t$ is $n(s,t)\ge 0$.
- The population is scaled to ensure $\sum_{s\in S} n(s,t)=N$ constant.
- From every generation to the next, every agent plays every other agent.
- Each agent's fitness is proportional to their gains, and the next generation's population is created with each agent scaled by their fitness.
  - The gains are summed in $g(s,t)$.
  - We would expect $\mathbb{E}[n(s,t+1)]\propto n(s,t)\cdot g(s,t)$.

## The Vanilla Strategies

Nicky Case's website includes these "vanilla" strategies:

- **Nice** – I always collaborate.

- **Evil** – I always defect..

- **Grudger** – I start Nice, but it takes a single betrayal for me to always defect.

- **Copycat** – I defect if and only if they have just defected.

- **Copykitten** – I start defecting if and only if they have just defected twice in a row. NOTE: it is up to the implementation how long of a "clean streak" the partner needs for this strategy to become nice again.

- **Detective** – I start with `[C,D,C,C]`; if the opponent responds with `[_,_,D,_]`, they are probably capable of retaliating, so I will play as Copycat; otherwise, I play as Evil.

- **Pavlov** – I start Nice, and I'm only ever Nice or Evil; if my last play coincided with opponent defection, I consider it karma and switch to the other one. NOTE: Pavlov is a simple RL learner with exactly 1 turn of memory and a boolean value table.

## The Parable

The way the story usually goes, Copycat takes the win under $\epsilon=0$, while Copykitten's forgiveness excels at small nonzero $\epsilon$'s. Being too Nice dooms you, but being too Evil ends up not viable either; the Grudger is heavily punished under a positive $\epsilon$ for having no forgiveness. Even an adaptive strategy like the Detective ends up slowly losing from their single defection.

(Meanwhile, it has been noted that in a population initialized with many Evil strategies, even Copycats can slowly lose from their initial niceness.)

## The Cracks In The Narrative

A simple list of how things can go very south from your expectations.

- With $\epsilon > 0$ and a large initial population of Nice, Grudger can briefly raise to dominance, not by punishing foul play, but by extorting the true altruists.
- With $\epsilon > 0$, a strategy that mindlessly alternates between collaboration and defection in a Copycat-dominated environment performs just as well.
- My friend Nyphakosi presented a custom strategy, the "green-beard altruist" who authenticates each other with an 8-bit key and enforces strict ingroup favoratism and outward hostility. This strategy easily rises to dominance with a slightly larger initial population than other strategies, which alarmingly resembles the ingroup favoratism displayed by Abrahamaic religions.
- (as of 2026-09-23) With $\epsilon > 0$, the presence of my custom strategy, the Businessman, enters an oscillating dynamic with the Copycat, while the Copykitten dies out.

## The Analysis That Birthed The Businessman

**Payout Is A Function, Not A Table.** We attach a boolean value to the actions played: 1 for Collaboration, 0 for Defection. Our move is always $u$, and their move is always $v$. We assume our payout per turn is a function of the actions, i.e., $f(u,v)=w+su+tv+yuv$ with $w,s,t,y\in\mathbb{R}$. Then, we are able to compute our relative payouts when we increase $u$ as $f(1,v)-f(0,v)=s+yv$, as well as that of our partner, $f(u,1)-f(u,0)=t+yu$.

**Reciprocity Is Delayed Gain.** We assume our partner has probability $p$ to collaborate, $q$ to defect, and $r$ to <u>reciprocate</u> our last move. This is in lieu of the success of the Copycat, but as we will see, the consequence of this assumption is a strategy *unlike* Copycat. We assume $p+q+r=1$; as a result, our expected immediate relative gain (delay=0) is $\mathrm{rg}_0=s+yv$. Since the next round there is chance $r$ our own move turns around to affect ourselves as $v$, our expected relative gain with delay=1 is $\mathrm{rg}_1=rt+yu'$ where $u'$ is our move in the next round.

**Linear Assumptions Save Us From Time-Traveling.** Predicting the opponent's move or even our own move in the next round is impractical; we note they all originate in the $+yuv$ cross-term within $f(u,v)$. Here we make a daring choice of setting $y=0$, so payout becomes $f(u,v)=w+su+tv$, and our relative gain from collaborating *instead of* defecting takes the following shape:
$$
\sum_i\mathrm{rg}_i=\mathrm{rg}_0+\mathrm{rg}_1=s+rt
$$
**Relative Gain Determines Our Move.** If $\sum\mathrm{rg}>0$ then we're expected to gain more by collaborating; otherwise we're expected to gain more from defecting. The catch: under linear assumptions, $\sum\mathrm{rg}=s+rt$ take on a closed form. If we can deduce $r,s,t$, then our move becomes uniquely determined. And it is NOT to play Copycat, but to play Nice against a Copycatl, and play Evil against anyone else.

**Computing Payout Coefficients $s$ and $t$.** We expect $f(0,0)=d$, $f(0,1)=c$, $f(1,0)=b$, $f(1,1)=a$; this gives rise to the following equations:
$$
\begin{align*}
w+s+t &= a \;, \\
w+s\quad\;\,\, &= b \;, \\
w\quad\;\:\,+t &= c \;, \\
w\quad\;\:\,\quad\;\,\, &= d \;.
\end{align*}
$$
The good news: the equations are fully determined. The bad news: the equations are over-determined. The good news: we can perform a least-square approximation of the solution:
$$
A^\top A\hat x=A^\top\vec b;\quad A={\scriptsize\begin{bmatrix}1&1&1 \\ 1&1&0 \\ 1&0&1 \\ 1&0&0\end{bmatrix}},\,\, \vec b={\scriptsize\begin{bmatrix}a\\b\\c\\d\end{bmatrix}},\,\, \hat x={\scriptsize\begin{bmatrix}w\\s\\t\end{bmatrix}}
$$
We solve this:
$$
\hat x=\frac14{\small\begin{bmatrix}-1&1&1&3\\2&2&-2&-2\\2&-2&2&-2\end{bmatrix}}\vec b
$$
**Estimating Reciprocity $r$.** Since we are estimating reciprocity with delay=1, we group our moves $u_t$ with opponent moves $v_{t+1}$, and count both totals $n_u$ binned by $u$, and coincidence $m_u$ where $u_t=v_{t+1}$ (also binned). We expect $\hat\mu_0=m_0/n_0 \to q+r$ and $\hat\mu_1=m_1/n_1 \to p+r$, and from our assumptions, $p+q+r=1$. Hence we have $\hat r=\hat\mu_0+\hat\mu_1-1$. Since we assume opponent decisions themselves are mutually independent, the variances are given by $\mathrm{Var}[r]=\mathrm{Var}[\mu_0]+\mathrm{Var}[\mu_1]$ where for $u$, $\mathrm{Var}[\mu_u]\gets\hat\mu_u(1-\hat\mu_u)/n_u$.

**Addressing Uncertainty With Active Exploration.** We take a confidence interval $\hat r\pm Z\hat\sigma_r$ where $Z>0$ is a constant we set. If $s+\hat r_\min t>0$, we play Nice; if $s+\hat r_\max t<0$, we play Evil. Otherwise, we seem to be left in limbo. Would having a more balanced dataset have reduced our $\hat\sigma_r$? So we compute a $\hat\sigma_{r,\text{ideal}}$ assuming the same $\hat\mu_0,\hat\mu_1$ but with $n_{0,\text{ideal}}=n_{1,\text{ideal}}=(n_0+n_1)/2$. We pick a constant $B>1$; if $\hat\sigma_r/\hat\sigma_{r,\text{ideal}}>B$, then we play whatever move we have played less of, judging by the current $n_0,n_1$; otherwise, we play Copycat out of caution.

**Interpretation.** We respect the sovereign, extort the vulnerable, and avoid the depraved. But all we see is expected relative gain. Is profit. This is what we have become, the statistician is a businessman. Shrewd, but cold.