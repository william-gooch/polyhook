pub const EXAMPLES: &[(&str, &str)] = &[
    ("Flat", EXAMPLE_FLAT),
    ("Spiral Rounds", EXAMPLE_SPIRAL_ROUNDS),
    ("Joined Rounds", EXAMPLE_JOINED_ROUNDS),
    ("Sphere", EXAMPLE_SPHERE),
    ("Dynamic Circle", EXAMPLE_DYNAMIC_CIRCLE),
    ("Sew", EXAMPLE_SEW),
    ("Bear", EXAMPLE_BEAR),
    ("Snowman", EXAMPLE_SNOWMAN),
];

pub const EXAMPLE_FLAT: &str = r#"rep 15 chain();
rep 15 {
    turn();
    rep 15 dc();
}
"#;

pub const EXAMPLE_SPIRAL_ROUNDS: &str = r#"fn inc() {
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
"#;

pub const EXAMPLE_JOINED_ROUNDS: &str = r#"fn inc() {
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
"#;

pub const EXAMPLE_SPHERE: &str = r#"fn inc() {
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
"#;

pub const EXAMPLE_DYNAMIC_CIRCLE: &str = include_str!("../examples/dynamic_circle.ph");
pub const EXAMPLE_SEW: &str = include_str!("../examples/sew.ph");
pub const EXAMPLE_BEAR: &str = include_str!("../examples/bear.ph");
pub const EXAMPLE_SNOWMAN: &str = include_str!("../examples/snowman.ph");
