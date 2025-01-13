fn inc() {
    dc_();
    dc();
}

magic_ring();
into(mark());
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