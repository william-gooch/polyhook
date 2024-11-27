pub const EXAMPLES: &[(&str, &str)] = &[
    ("Flat", EXAMPLE_FLAT),
    ("Spiral Rounds", EXAMPLE_SPIRAL_ROUNDS),
    ("Joined Rounds", EXAMPLE_JOINED_ROUNDS),
    ("Sphere", EXAMPLE_SPHERE),
    ("Dynamic Circle", EXAMPLE_DYNAMIC_CIRCLE),
];

pub const EXAMPLE_FLAT: &str = r#"15 # chain;
15 # || {
    turn();
    15 # dc;
}
"#;

pub const EXAMPLE_SPIRAL_ROUNDS: &str = r#"fn inc() {
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

20 @ |j| {
    6 # || {
        j # dc;
        inc();
    }
}
"#;

pub const EXAMPLE_JOINED_ROUNDS: &str = r#"fn inc() {
    dc_();
    dc();
}

let start = mark();
5 # chain;
ss(start);

turn_();
let start = mark();
dc();
5 # inc;
ss(start);

20 @ |round| {
    turn();
    let start = mark();
    6 # || {
        inc();
        round # dc;
    };
    ss(start);
}
"#;

pub const EXAMPLE_SPHERE: &str = r#"fn inc() {
    dc_();
    dc();
}

let ch_sp = chain_space(|| {
    let start = mark();
    2 # chain;
    ss(start);
});

into(ch_sp);
let start = mark();
5 # dc_;
into(start);

4 @ |j| {
    6 # || {
        j # || {
            dc();
        };
        inc();
    };
};

7 # || {
    36 # dc;
};

4 @ |j| {
    6 # || {
        (5-j) # dc;
        dec();
    };
};

2 # dec;
"#;

pub const EXAMPLE_DYNAMIC_CIRCLE: &str = include_str!("../examples/dynamic_circle.ph");
