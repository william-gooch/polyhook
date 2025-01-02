fn inc() {
    dc_();
    dc();
}

let ch_sp = chain_space(|| {
    let start = mark();
    rep 2 chain();
    ss(start);
});

new_row();
into(ch_sp);
rep 6 dc_();

let j = 1;
rep 5 {
    new_row();
    rep 6 {
        rep (j-1) dc();
        inc();
    };
    j += 1;
};

rep 7 {
    new_row();
    rep 36 dc();
};

let j = 1;
rep 5 {
    new_row();
    rep 6 {
        rep (5-j) dc();
        dec();
    };
    j += 1;
};

new_row();
rep 2 dec();