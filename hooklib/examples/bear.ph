fn bobble() {
	ignore(|| { rep 4 dc_() });
	dc();
}

fn inc() {
	dc_();
	dc();
}

magic_ring();

// row 1
new_row();
rep 6 dc_();

// row 2
new_row();
rep 6 {
	dc_();
	dc();
};

// row 3
new_row();
dc();
inc();
bobble();
inc();
rep 2 {
	dc();
	inc();
};
bobble();
inc();
dc();
inc();

// row 4-6
rep 3 {
	new_row();
	rep 18 dc();
};

// row 7
new_row();
rep 6 {
	dc();
	dec();
};

// row 8
new_row();
rep 12 dc();

// row 9
new_row();
rep 4 dc();
bobble();
rep 3 dc();
bobble();
rep 3 dc();

// row 10
new_row();
rep 12 dc();

// row 11
new_row();
rep 4 dc();
bobble();
rep 3 dc();
bobble();
rep 3 dc();

// row 12
new_row();
chain_space(|| {
    rep 4 {
        dc();
        dec();
    }
})