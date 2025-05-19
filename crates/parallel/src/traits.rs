pub trait Convertable {
    fn to_vec(self) -> Vec<usize>;
    fn from_vec(v: &[usize]) -> Self;
}