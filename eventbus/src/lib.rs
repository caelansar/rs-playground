#![feature(fn_traits)]

mod macros;
use anymap::{AnyMap, Entry};
use std::collections::HashMap;
use std::fmt::Display;
use std::mem;

pub struct Request<T> {
    data: T,
}

impl<T> Request<T> {
    fn new(data: T) -> Self {
        Self { data }
    }
}

trait FromRequest<T> {
    fn from_request(req: &Request<T>) -> Self;
}

pub trait Handler<T> {
    fn call(&self, req: Request<T>);
}

impl<F> Handler<()> for F
where
    F: FnOnce() + Copy,
{
    fn call(&self, _: Request<()>) {
        (self)()
    }
}

impl<'a> FromRequest<&'a str> for &'a str {
    fn from_request(req: &Request<&'a str>) -> Self {
        req.data
    }
}

impl<'a, T1> FromRequest<&'a T1> for &'a T1
where
    T1: FromRequest<T1> + Copy,
{
    fn from_request(req: &Request<&'a T1>) -> Self {
        req.data
    }
}

impl<T1: Copy, const N: usize> FromRequest<[T1; N]> for [T1; N] {
    fn from_request(req: &Request<[T1; N]>) -> Self {
        req.data
    }
}

impl<'a, T1> FromRequest<&'a [T1]> for &'a [T1]
where
    T1: FromRequest<T1> + Copy,
{
    fn from_request(req: &Request<&'a [T1]>) -> Self {
        req.data
    }
}

#[repr(C)]
struct TraitObject {
    pub data: *mut (),
    pub vtable: *mut (),
}

pub struct EventBus {
    handlers: AnyMap,
}

#[allow(dead_code)]
struct HandlerPtr<T> {
    handler: Box<dyn Handler<T>>,
    trait_object: TraitObject,
}

impl<T> HandlerPtr<T> {
    fn new(handler: Box<dyn Handler<T>>) -> Self {
        let trait_object: TraitObject = unsafe { mem::transmute(&*handler) };
        HandlerPtr {
            handler,
            trait_object,
        }
    }
}

type HandlerMap<T> = HashMap<String, HandlerPtr<T>>;

impl EventBus {
    pub fn new() -> EventBus {
        EventBus {
            handlers: AnyMap::new(),
        }
    }

    pub fn subscribe<T: 'static, H: Handler<T> + 'static>(&mut self, topic: String, handler: H) {
        let handler_ptr = HandlerPtr::new(Box::new(handler));
        match self.handlers.entry::<HandlerMap<T>>() {
            Entry::Occupied(inner) => {
                inner.into_mut().insert(topic, handler_ptr);
            }
            Entry::Vacant(inner) => {
                let mut h = HashMap::new();
                h.insert(topic, handler_ptr);
                inner.insert(h);
            }
        }
        // println!("subscribe T: {}", std::any::type_name::<T>());
    }

    pub fn publish<T: 'static>(&self, topic: impl AsRef<str> + Display, arg: Request<T>) {
        self.dispatch(topic, arg);
    }

    fn dispatch<T: 'static>(&self, topic: impl AsRef<str> + Display, arg: Request<T>) {
        if let Some(handlers) = self.handlers.get::<HandlerMap<T>>() {
            handlers
                .get(topic.as_ref())
                .ok_or("handler not found")
                .and_then(|x| {
                    x.handler.call(arg);
                    Ok(())
                })
                .unwrap();
        } else {
            panic!("handler not found")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eventbus_works() {
        let mut bus = EventBus::new();
        bus.subscribe("topic0".to_string(), |x: &i32| {
            println!("closure accept &str: {x}")
        });
        bus.subscribe("topic1".to_string(), |x: i32, y: i32| {
            println!("closure accept i32: {x}, i32: {y}")
        });
        bus.subscribe("topic2".to_string(), |x: i32, y: &str, z: i32| {
            println!("closure accept i32: {x}, &str: {y}, i32: {z}")
        });
        bus.subscribe("topic3".to_string(), || println!("closure have no param"));

        bus.subscribe("topic4".to_string(), |x: [u8; 2]| {
            println!("closure accept [u8]: {x:?}")
        });

        bus.subscribe("topic5".to_string(), foo);

        bus.publish("topic0", new_request!(&1i32));
        bus.publish("topic1", new_request!(1i32, 2i32));
        bus.publish("topic2", new_request!(1i32, "100", 2i32));
        bus.publish("topic3", new_request!());
        bus.publish("topic4", new_request!([1u8, 2]));
        bus.publish("topic5", new_request!(&[1i32, 2][..]));
    }

    fn foo(x: &[i32]) {
        println!("fn accept &[i32]: {x:?}")
    }
}
