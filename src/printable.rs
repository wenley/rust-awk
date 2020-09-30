//
// Proper Monad implementing the monadic laws
//
struct Printable<T> {
    value: T,
    output: Vec<String>,
}

impl<T> Printable<T> {
    pub(crate) fn wrap(value: T) -> Printable<T> {
        Printable {
            value: value,
            output: vec![],
        }
    }

    pub(crate) fn and_then<U, F>(self, f: F) -> Printable<U>
    where
        F: FnOnce(T) -> Printable<U>,
    {
        let Printable { value, mut output } = self;
        let Printable {
            value: new_value,
            output: mut new_output,
        } = f(value);
        output.append(&mut new_output);

        Printable {
            value: new_value,
            output: output,
        }
    }
}
