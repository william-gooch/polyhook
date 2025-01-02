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
chain();
rep 6 dc_();

new_row();
rep 6 {
    inc();
};

let j = 1;
rep 20 {
    new_row();
    rep 6 {
        rep j dc();
        inc();
    };
    j += 1;
};