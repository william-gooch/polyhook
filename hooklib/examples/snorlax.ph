// Pattern (c) Shelly Hedko
// Available at: https://epic-yarns.com/2012/01/23/snorlax/
// Creative Commons BY-NC-ND http://creativecommons.org/licenses/by-nc-nd/3.0/

fn inc() { dc_(); dc(); }

const blue = [0.15, 0.45, 0.55];
const beige = [0.95, 0.9, 0.85];
const white = [1.0, 1.0, 1.0];

fn body() {
    change_color(global::blue);

    magic_ring();
    into(mark());
    rep 6 dc_();

    for i in 0..=8 {
        new_row();
        rep 6 { rep i dc(); inc(); };
    }

    rep 3 {
        new_row();
        rep 60 dc();
    };

    let rs = [];
    rep 5 {
        new_row();
        rep 60 dc();
        rs.push(row());
    };
    
    let head_sew = [
        rs[0][29], rs[0][30], rs[0][31],
        rs[1][31], rs[2][31], rs[3][31],
        rs[4][31], rs[4][30], rs[4][29],
        rs[3][29], rs[2][29], rs[1][29],
    ];

    let arm_1_sew = [
        rs[0][19], rs[0][20], rs[0][21], rs[0][22], rs[0][23],
        rs[1][23], rs[2][23], rs[3][23],
        rs[4][22], rs[4][21], rs[4][20], rs[4][19],
        rs[3][19], rs[2][19], rs[1][19],
    ];

    let arm_2_sew = [
        rs[0][39], rs[0][40], rs[0][41], rs[0][42], rs[0][43],
        rs[1][43], rs[2][43], rs[3][43],
        rs[4][42], rs[4][41], rs[4][40], rs[4][39],
        rs[3][39], rs[2][39], rs[1][39],
    ];
    arm_2_sew.reverse();

    let foot_1_sew = [rs[2][5], rs[2][6], rs[2][7], rs[3][7], rs[3][6], rs[3][5]];
    let foot_2_sew = [rs[2][-5], rs[2][-6], rs[2][-7], rs[3][-7], rs[3][-6], rs[3][-5]];

    rep 2 {
        new_row();
        rep 60 dc();
    };

    new_row();
    rep 6 { dec(); rep 8 dc(); };
    let belly_sew = row();

    for i in 1..=8 {
        new_row();
        rep 6 { dec(); rep (8 - i) dc(); };
    }

    new_part();
    let arm_1 = arm();
    new_part();
    let arm_2 = arm();

    sew(arm_1_sew, arm_1);
    sew(arm_2_sew, arm_2);

    new_part();
    let foot_1 = foot();
    new_part();
    let foot_2 = foot();

    sew(foot_1_sew, foot_1);
    sew(foot_2_sew, foot_2);

    [belly_sew, head_sew]
}

fn belly() {
    change_color(global::beige);

    magic_ring();
    into(mark());
    rep 6 dc_();

    for i in 0..=7 {
        new_row();
        rep 6 { rep i dc(); inc(); };
    }

    row();
}

fn face() {
    change_color(global::beige);

    magic_ring();
    into(mark());
    rep 8 dc_();

    new_row();
    rep 8 { inc(); };

    turn_();
    rep 2 dc();
    inc();
    rep 2 dc();
    inc();
    rep 4 dc();
    inc();
    rep 2 dc();
    inc();
    rep 2 dc();

    turn_();
    rep 20 dc();

    row().extract(1);
}

fn ear() {
    change_color(global::blue);

    magic_ring();
    into(mark());
    rep 3 dc_();

    new_row();
    rep 3 { inc(); };
    new_row();
    rep 2 { rep 2 dc(); inc(); };
    new_row();
    rep 2 { rep 3 dc(); inc(); };

    row()
}

fn head() {
    change_color(global::blue);

    magic_ring();
    into(mark());
    rep 6 dc_();

    let rs = [];
    for i in 0..=3 {
        new_row();
        rep 6 {
            rep i dc(); inc();
        };
        rs.push(row());
    }

    rep 5 {
        new_row();
        rep 30 dc();
        rs.push(row());
    };

    for i in 0..=2 {
        new_row();
        rep 6 { dec(); rep (3 - i) dc(); };
        rs.push(row());
    }

    let ear_1_sew = [rs[1][0], rs[1][1], rs[2][2], rs[3][2], rs[4][2], rs[5][1], rs[5][0], rs[4][-1], rs[3][-1], rs[2][-1]];
    let ear_2_sew = [rs[1][6], rs[1][7], rs[2][10], rs[3][17], rs[4][17], rs[5][16], rs[5][15], rs[4][14], rs[3][14], rs[2][10]];

    let face_sew = [
         rs[5][4],  rs[5][5],  rs[5][6],  rs[5][7],  rs[5][8],
         rs[5][9],  rs[6][9],  rs[7][9],  rs[8][9],  rs[9][9],
         rs[9][7], rs[9][6], rs[9][5], rs[9][4], rs[9][3],
         rs[9][3],  rs[8][3],  rs[7][3],  rs[6][3],  rs[5][3],
    ];
    face_sew = face_sew.extract(2..face_sew.len) + face_sew.extract(0..2);

    let head_sew = row();
    head_sew = head_sew.extract(7..head_sew.len) + head_sew.extract(0..7);

    new_row();
    rep 6 { dec(); };

    new_part();
    let face = face();
    new_part();
    let ear_1 = ear();
    new_part();
    let ear_2 = ear();

    sew(face_sew, face);
    sew(ear_1_sew, ear_1);
    sew(ear_2_sew, ear_2);

    head_sew
}

fn arm() {
    change_color(global::blue);

    magic_ring();
    into(mark());
    rep 8 dc_();

    new_row(); rep 8 dc();
    new_row(); rep 2 { rep 3 dc(); inc(); };
    new_row(); rep 10 dc();
    new_row(); rep 2 { rep 4 dc(); inc(); };
    rep 4 { new_row(); rep 12 dc(); };
    new_row(); rep 3 { rep 3 dc(); inc(); };
    new_row(); rep 15 dc();
    let sew_1 = row().extract(0..7);
    sew_1.reverse();
    turn_(); rep 8 dc();
    let sew_2 = row().extract(1);

    sew_1 + sew_2
}

fn foot() {
    change_color(global::beige);

    magic_ring();
    into(mark());
    rep 8 dc_();

    new_row(); rep 8 inc();
    new_row(); dc(); inc(); dc(); inc(); rep 12 dc();
    new_row(); rep 18 dc();
    new_row(); rep 3 { rep 4 dc(); dec(); };
    new_row(); rep 3 { rep 3 dc(); dec(); };
    new_row(); rep 6 { dec(); };

    row()
}

let body_sews = body();
new_part();
let belly_sew = belly();
new_part();
let head_sew = head();

sew(body_sews[0], belly_sew);
sew(body_sews[1], head_sew);
