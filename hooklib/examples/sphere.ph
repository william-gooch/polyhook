// Pattern from 'The Ideal Crochet Sphere' by Ms Premise-Conclusion
// Available at: https://mspremiseconclusion.wordpress.com/2010/03/14/the-ideal-crochet-sphere/

fn inc() {
    dc_();
    dc();
}

fn inc_sequence(list) {
    for i in 0..(list.len() - 1) {
        rep list[i] dc();
        inc();
    }
    rep list[-1] dc();
}

fn dec_sequence(list) {
    rep list[0] dc();
    for i in 1..list.len() {
        dec();
        rep list[i] dc();
    }
}

magic_ring();
into(mark());
rep 6 dc_();

new_row(); rep 6 inc();
new_row(); inc_sequence([1, 2, 1, 2, 1, 0]); 
new_row(); inc_sequence([2, 2, 1, 2, 2, 1, 1]); 
new_row(); inc_sequence([3, 5, 5, 5, 1]); 
new_row(); inc_sequence([4, 6, 6, 6, 1]); 
new_row(); inc_sequence([8, 9, 9, 2]); 
new_row(); inc_sequence([10, 16, 6]); 
new_row(); inc_sequence([17, 18]); 
new_row(); rep 37 dc();
new_row(); dec_sequence([18, 17]);
new_row(); dec_sequence([6, 16, 10]);
new_row(); dec_sequence([2, 9, 9, 8]);
new_row(); dec_sequence([1, 6, 6, 6, 4]);
new_row(); dec_sequence([1, 5, 5, 5, 3]);
new_row(); dec_sequence([1, 1, 2, 2, 1, 2, 2]);
new_row(); dec_sequence([0, 1, 2, 1, 2, 1]);
new_row(); rep 6 dec();