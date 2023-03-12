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
    #[allow(unused_labels)]
    'y: {
        let x: usize = 10;
        b.do_something(&x);
    }
}

struct Closure<F> {
    data: (u8, u16),
    func: F,
}
impl<F> Closure<F>
where
    for<'a> F: Fn(&'a (u8, u16)) -> &'a u8,
{
    fn call(&self) -> &u8 {
        (self.func)(&self.data)
    }
}
fn do_it(data: &(u8, u16)) -> &u8 {
    &data.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closure_should_work() {
        #[allow(unused_labels)]
        'x: {
            let clo = Closure {
                data: (0, 1),
                func: do_it,
            };
            println!("{}", clo.call());
        }
    }

    #[test]
    fn do_something_should_works() {
        let s = "aa";
        (&s).do_something("bb");

        #[allow(unused_labels)]
        'x: {
            bar(1usize);
        }
    }

    #[test]
    fn option_filter_works() {
        let v = Some(1);
        assert_eq!(Some(1), v.filter1(|x| *x > 0));
        assert_eq!(None, v.filter1(|x| *x < 0));
    }
}
