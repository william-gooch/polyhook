fn inc() {
    dc_();
    dc();
}

let ch_sp = chain_space(|| {
    let start = mark();
    2 # chain;
    ss(start);
});

into(ch_sp);
chain();
let start = mark();
5 # dc_;

into(start);
6 # || {
    inc();
};

let round_size = 12.0;
let round_target = 12.0;
15 @ |j| {
	round_target += 6.28 * (15.0 / 18.5);
	let incs = floor(round_target - round_size).to_int();
	let steps = floor(round_size / incs).to_int();
	let rem = (round_size % incs).to_int();

	let round_c = 0.0;
	for i in 1..=incs {
        (steps - 1) # dc;
        inc();
		round_c += steps + 1;
	}
	rem # dc;
	round_size = round_c;
}
