fn inc() {
    dc_();
    dc();
}

let ch_sp = chain_space(|| {
    let start = mark();
    rep 2 chain();
    ss(start);
});

into(ch_sp);
rep 6 dc_();

new_row();
rep 6 {
    inc();
};

let round_size = 12;
let round_target = 12.0;
rep 15 {
	new_row();

	round_target += 6.28 * (15.0 / 18.5);
	let incs = round(round_target).to_int() - round_size;
	let steps = round_size / incs;
	let rem = round_size % incs;


	let round_c = 0;
	for i in 1..=incs {
        rep (steps - 1) dc();
        inc();
		round_c += steps + 1;
	}
	rep rem dc();
	round_c += rem;
	round_size = round_c;
}
