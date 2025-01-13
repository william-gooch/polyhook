// Pattern (c) Alexandra Halsey
// Available at: https://www.withalexofficialblog.com/2018/06/crescent-moon-free-pattern.html
// Creative Commons Attribution ShareAlike License https://creativecommons.org/licenses/by-sa/3.0/

magic_ring();
into(mark());
rep 8 dc_();

new_row();

rep 8 { dc_(); dc(); };
new_row();
rep 8 { dc_(); dc(); rep 1 dc(); };
new_row();
rep 8 { dc_(); dc(); rep 2 dc(); };
new_row();
rep 8 { dc_(); dc(); rep 3 dc(); };
new_row();
rep 8 { dc_(); dc(); rep 4 dc(); };
new_row();
rep 8 { dc_(); dc(); rep 5 dc(); };

let r = row();
let half_1 = r.extract(0..(r.len/2));
let half_2 = r.extract((r.len/2)..r.len);
half_2.reverse();

sew(half_1, half_2);