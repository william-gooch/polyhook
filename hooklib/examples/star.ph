// Pattern (c) Alexandra Halsey
// Available at: https://www.withalexofficialblog.com/2018/05/amigurumi-star-free-crochet-pattern.html
// Creative Commons Attribution ShareAlike License https://creativecommons.org/licenses/by-nc-nd/3.0/

fn star_half() {
    magic_ring();
    into(mark());

    let start = mark();
    rep 10 dc_();

    new_row();
    rep 5 { dc_(); dc(); dc(); };

    new_row();
    rep 5 {
        rep 5 dc_();
        dec();
    };

    row()
}

let row_1 = star_half();
new_part();
let row_2 = star_half();

sew(row_1, row_2);