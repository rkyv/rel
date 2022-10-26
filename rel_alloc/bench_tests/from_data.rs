pub struct FromData<'a, R, T> {
    pub alloc: R,
    pub data: &'a T,
}
