pub trait Convertable {
    fn to_vec(self) -> Vec<u64>;
    fn add_from_vec(&mut self, v: &[u64]);
}
pub trait ParallelRun {
    fn run(self);
}