//
// Proper Monad implementing the monadic laws
//
pub(crate) struct Printable<T> {
    pub(crate) value: T,
    pub(crate) output: Vec<String>,
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

    pub(crate) fn map<U, F>(self, f: F) -> Printable<U>
    where
        F: FnOnce(T) -> U,
    {
        let Printable { value, output } = self;
        Printable {
            value: f(value),
            output: output,
        }
    }
}
