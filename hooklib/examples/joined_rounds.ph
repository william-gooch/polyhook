fn inc() {
    dc_();
    dc();
}

let start = mark();
rep 5 chain();
ss(start);

turn_();
let start = mark();
rep 6 inc();
ss(start);

let round = 1;
rep 20 {
    turn();
    let start = mark();
    rep 6 {
        inc();
        rep round dc();
    };
    ss(start);
    round += 1;
}