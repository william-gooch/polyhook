fn bobble() {
	ignore(|| { 4 # dc_ });
	dc();
}

fn inc() {
	dc_();
	dc();
}

let start = mark();
2 # chain;

// row 1
new_row();
into(start);
6 # dc_;

// row 2
new_row();
6 # || {
	dc_();
	dc();
};

// row 3
new_row();
dc();
inc();
bobble();
inc();
2 # || {
	dc();
	inc();
};
bobble();
inc();
dc();
inc();

// row 4-6
3 # || {
	new_row();
	18 # dc;
};

// row 7
new_row();
6 # || {
	dc();
	dec();
};

// row 8
new_row();
12 # dc;

// row 9
new_row();
4 # dc;
bobble();
3 # dc;
bobble();
3 # dc;

// row 10
new_row();
12 # dc;

// row 11
new_row();
4 # dc;
bobble();
3 # dc;
bobble();
3 # dc;

// row 12
new_row();
chain_space(|| {
    4 # || {
        dc();
        dec();
    }
})