fn inc() {
	let s1 = dc_();
	let s2 = dc();
	[s1, s2]
}

fn body() {
magic_ring();
into(mark());
8 # dc_;

new_row();
8 # inc;
let hat_sews = row();

new_row();
8 # || {
	dc();
	inc();
};

new_row();
6 # || {
	3 # dc;
	inc();
};

new_row();
30 # dc;


new_row();
6 # || {
	4 # dc;
	inc();
};

let nose_sews = [
	row()[14],
	row()[15],
];


new_row();
36 # dc;

let last_sew = row()[13];
nose_sews.push(row()[16]);

new_row();
36 # dc;

nose_sews.push(row()[15]);
nose_sews.push(row()[14]);
nose_sews.push(last_sew);

new_row();
36 # dc;

new_row();
36 # dc;

new_row();
6 # || {
	4 # dc;
	dec();
};

new_row();
6 # || {
	3 # dc;
	dec();
};

new_row();
4 # || {
	4 # dc;
	dec();
};

new_row();
10 # || {
	dc();
	inc();
};

new_row();
6 # || {
	4 # dc;
	inc();
};

new_row();
6 # || {
	5 # dc;
	inc();
};

new_row();
3 # || {
	13 # dc;
	inc();
};

4 # || {
	new_row();
	45 # dc;
};

new_row();
3 # || {
	13 # dc;
	dec();
};

new_row();
6 # || {
	5 # dc;
	dec();
};

new_row();
6 # || {
	4 # dc;
	dec();
};

new_row();
6 # || {
	3 # dc;
	dec();
};

new_row();
8 # || {
	dc();
	dec();
};

new_row();
8 # dec;

[hat_sews, nose_sews]
}

fn hat() {
magic_ring();
into(mark());

8 # dc_;

new_row();
8 # inc;

new_row();
8 # || {
	dc();
	inc();
};

new_row();
24 # dc;

new_row();
4 # || {
	4 # dc;
	dec();
};

new_row();
20 # dc;

new_row();
4 # || {
	3 # dc;
	dec();
};

new_row();
16 # dc;
let body_sews = row();

new_row();
8 # || {
	dc();
	inc();
};

new_row();
8 # || {
	2 # dc;
	inc();
};

new_row();
dc();
ss(curr());

body_sews
}

fn nose() {
magic_ring();
into(mark());
4 # dc_;

new_row();
2 # || {
	dc();
	inc();
};

new_row();
6 # dc;

row()
}

let body_sews = body();
new_part();
let hat_sews = hat();
new_part();
let nose_sews = nose();

sew(body_sews[0], hat_sews);
sew(body_sews[1], nose_sews);