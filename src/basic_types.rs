
pub enum Field {
    WholeLine,
    Indexed(usize),
}

pub struct Record<'a> {
    pub full_line: &'a str,
    pub fields: &'a Vec<&'a str>,
}

pub struct Context {
}

