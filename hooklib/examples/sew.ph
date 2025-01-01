rep 15 chain();
rep 14 {
    turn();
    rep 15 dc();
};

turn();
let row_1 = [];
rep 15 {
	dc();
	row_1.push(mark());
};

new_part();

rep 15 chain();
rep 14 {
    turn();
    rep 15 dc();
};

turn();
let row_2 = [];
rep 15 {
	dc();
	row_2.push(mark());
};

sew(row_1, row_2)