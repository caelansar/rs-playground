trait OptionFilter<T> {
    fn filter1<F>(self, f: F) -> Option<T>
    where
        F: for<'a> FnOnce(&'a T) -> bool;
}

impl<T> OptionFilter<T> for Option<T> {
    fn filter1<F>(self, f: F) -> Option<T>
    where
        F: for<'a> FnOnce(&'a T) -> bool,
    {
        if let Some(x) = self {
            if f(&x) {
                return Some(x);
            }
        }
        None
    }
}

trait Trait<T> {
    fn do_something(&self, value: T);
}

impl<'a, T: ?Sized> Trait<&'a T> for &'a T {
    fn do_something(&self, value: &'a T) {
        println!("do something for &'a T")
    }
}

impl<'a, T: ?Sized> Trait<&'a T> for T {
    fn do_something(&self, value: &'a T) {
        println!("do something for T")
    }
}

// could not compile
//fn foo<'a>(b: impl Trait<&'a usize>) {
//    let x: usize = 10;
//    // x does not live long enough
//    //argument requires that `x` is borrowed for `'a`
//    b.do_something(&x);
//} // - `x` dropped here while still borrowed

fn bar(b: impl for<'a> Trait<&'a usize>) {
    let x: usize = 10;
    b.do_something(&x);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_something_should_works() {
        let s = "aa";
        (&s).do_something("bb");

        bar(1usize);
    }

    #[test]
    fn option_filter_works() {
        let v = Some(1);
        assert_eq!(Some(1), v.filter1(|x| *x > 0));
        assert_eq!(None, v.filter1(|x| *x < 0));
    }
}
