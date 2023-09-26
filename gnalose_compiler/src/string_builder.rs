use std::fmt::Display;

pub fn push(builder: &mut Vec<char>, v: &str) {
    builder.extend_from_slice(v.chars().collect::<Vec<char>>().as_slice());
}
pub fn push_line(builder: &mut Vec<char>, v: &str) {
    push(builder, format!("{}\n", v).as_str());
}
pub fn reduce_additive<'a, I, T, F>(slice: I, operation: F) -> String
where
    I: Iterator<Item = T>,
    F: Fn(T) -> String,
{
    let mut t: Vec<char> = Vec::new();
    for var in slice {
        push(&mut t, operation(var).as_str());
    }
    return collapse(t);
}
pub fn collapse(v: Vec<char>) -> String {
    return v.into_iter().collect();
}
pub fn bulk<T, It>(it: T) -> String
where
    T: Iterator<Item = It>,
    It: Display,
{
    return reduce_additive(it, |a| a.to_string());
}

pub struct Builder(pub Vec<char>);
impl Builder {
    pub fn new() -> Builder {
        return Builder(vec![]);
    }
    pub fn push(&mut self, t: &str) {
        push(&mut self.0, t);
    }
    pub fn push_line(&mut self, t: &str) {
        push_line(&mut self.0, t);
    }
    pub fn collapse(self) -> String {
        return collapse(self.0);
    }
}
