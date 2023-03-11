#[derive(Clone)]
pub struct Request;

pub trait Handler<T> {
    fn call(self, req: Request);
}

pub trait FromRequest {
    fn from_request(req: &Request) -> Self;
}

struct Param1(usize);
struct Param2(String);

macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!(T1);
        $name!(T1, T2);
        $name!(T1, T2, T3);
        $name!(T1, T2, T3, T4);
    };
}

macro_rules! impl_handler {
    (
        $($ty:ident),*
    ) => {
        #[allow(non_snake_case)]
        impl<F, $($ty,)*> Handler<($($ty,)*)> for F
        where
            F: FnOnce($($ty,)*),
            $( $ty: FromRequest, )*
        {
            fn call(self, req: Request) {
                $(
                    let $ty = $ty::from_request(&req);
                )*

                self($($ty,)*);
            }
        }
    };
}

all_the_tuples!(impl_handler);

// macro expand
// impl<F, T1> Handler<T1> for F
// where
//     F: FnOnce(T1),
//     T1: FromRequest,
// {
//     fn call(self, req: Request) {
//         (self)(T1::from_request(&req));
//     }
// }

// macro expand
// impl<F, T1, T2> Handler<(T1, T2)> for F
// where
//     F: FnOnce(T1, T2),
//     T1: FromRequest,
//     T2: FromRequest,
// {
//     fn call(self, req: Request) {
//         (self)(T1::from_request(&req), T2::from_request(&req));
//     }
// }

impl FromRequest for Param1 {
    fn from_request(req: &Request) -> Self {
        Param1(1)
    }
}

impl FromRequest for Param2 {
    fn from_request(req: &Request) -> Self {
        Param2("s".to_string())
    }
}

pub fn on<T, H>(req: Request, handler: H)
where
    H: Handler<T>,
{
    handler.call(req);
}

#[cfg(test)]
mod tests {
    use super::{on, Param1, Param2, Request};

    fn handler1(Param1(param1): Param1) {
        println!("param1: {param1}");
    }

    fn handler2(Param1(param1): Param1, Param2(param2): Param2) {
        println!("param1: {param1}, param2: {param2}");
    }

    #[test]
    fn test_axum_like_params() {
        let req = Request;
        on(req.clone(), handler1);
        on(req.clone(), handler2);
    }
}
