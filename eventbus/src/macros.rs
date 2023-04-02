use crate::{FromRequest, Handler, Request};

#[macro_export]
macro_rules! new_request {
    ($e:expr) => {{
        {
            Request::new(($e,))
        }
    }};


    ($($e:expr),*) => {{
        Request::new(($($e,)*))
    }};
}

macro_rules! all_primitive_type {
    ($name:ident) => {
        $name!(i8);
        $name!(i32);
        $name!(i64);
        $name!(i128);
        $name!(isize);
        $name!(u8);
        $name!(u32);
        $name!(u64);
        $name!(u128);
        $name!(usize);
        $name!(f32);
        $name!(f64);
        $name!(char);
        $name!(bool);
    };
}

macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!(T1);
        $name!(T1, T2);
        $name!(T1, T2, T3);
        $name!(T1, T2, T3, T4);
        $name!(T1, T2, T3, T4, T5);
        $name!(T1, T2, T3, T4, T5, T6);
        $name!(T1, T2, T3, T4, T5, T6, T7);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8);
    };
}

macro_rules! impl_handler {
    ($($ty:ident),*) => {
        impl<F, $($ty,)*> Handler<($($ty,)*)> for F
        where
            F: FnOnce($($ty,)*) + Copy,
            ($($ty,)*): FromRequest<($($ty,)*)>,
        {
            fn call(&self, req: Request<($($ty,)*)>) {
                let data: ($($ty,)*) = FromRequest::<($($ty,)*)>::from_request(&req);
                std::ops::FnOnce::call_once(*self, data)
            }
        }
    };
}

macro_rules! impl_from_request {
    ($ty:ident) => {
        impl FromRequest<$ty> for $ty {
            fn from_request(req: &Request<$ty>) -> Self {
                req.data
            }
        }
    };
}

macro_rules! impl_from_request_generic {
    ($($ty:ident),*) => {
        impl<$($ty,)*> FromRequest<($($ty,)*)> for ($($ty,)*)
        where
            $( $ty: Copy, )*
        {
            fn from_request(req: &Request<($($ty,)*)>) -> Self {
                req.data
            }
        }
    };
}

all_primitive_type!(impl_from_request);
all_the_tuples!(impl_from_request_generic);
all_the_tuples!(impl_handler);
