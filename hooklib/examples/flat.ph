export let width = 15;
export let height = 15;

rep width chain();
rep height {
    turn();
    rep width dc();
}