export let width = 5;
export let height = 10;

rep (width * 3 + 3) chain();
turn();

rep width {
    rep 3 skip();
    dc_();
    chain();
    dc_();
};

rep 2 skip();
dc();
rep 2 chain();
turn();

rep (height / 2) {
    change_color([0.3, 0.7, 0.0]);
    skip();
    rep width {
        rep 3 skip();
        dc_();
        chain();
        dc_();
    };

    skip();
    dc();
    rep 2 chain();
    turn();

    change_color([1.0, 1.0, 1.0]);
    skip();
    rep width {
        rep 3 skip();
        dc_();
        chain();
        dc_();
    };

    skip();
    dc();
    rep 2 chain();
    turn();
}