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

impl<F> Handler<i32> for F
where
    F: FnOnce(i32) + Copy,
{
    fn call(&self, req: Request<i32>) {
        let data = FromRequest::<i32>::from_request(&req);
        (self)(data)
    }
}

impl<F, T1, T2> Handler<(T1, T2)> for F
where
    F: FnOnce(T1, T2) + Copy,
    (T1, T2): FromRequest<(T1, T2)>,
{
    fn call(&self, req: Request<(T1, T2)>) {
        let data: (T1, T2) = FromRequest::<(T1, T2)>::from_request(&req);
        (self)(data.0, data.1)
    }
}

impl<F, T1, T2, T3> Handler<(T1, T2, T3)> for F
where
    F: FnOnce(T1, T2, T3) + Copy,
    (T1, T2, T3): FromRequest<(T1, T2, T3)>,
{
    fn call(&self, req: Request<(T1, T2, T3)>) {
        let data: (T1, T2, T3) = FromRequest::<(T1, T2, T3)>::from_request(&req);
        (self)(data.0, data.1, data.2)
    }
}

impl<T1, T2> FromRequest<(T1, T2)> for (T1, T2)
where
    T1: Copy,
    T2: Copy,
{
    fn from_request(req: &Request<(T1, T2)>) -> Self {
        req.data
    }
}

impl<T1, T2, T3> FromRequest<(T1, T2, T3)> for (T1, T2, T3)
where
    T1: Copy,
    T2: Copy,
    T3: Copy,
{
    fn from_request(req: &Request<(T1, T2, T3)>) -> Self {
        req.data
    }
}

impl<'a> FromRequest<&'a str> for &'a str {
    fn from_request(req: &Request<&'a str>) -> Self {
        req.data
    }
}

impl FromRequest<i32> for i32 {
    fn from_request(req: &Request<i32>) -> Self {
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
    }

    pub fn publish<T: 'static>(&self, topic: impl AsRef<str> + Display, arg: T) {
        self.dispatch(topic, arg);
    }

    fn dispatch<T: 'static>(&self, topic: impl AsRef<str> + Display, arg: T) {
        if let Some(handlers) = self.handlers.get::<HandlerMap<T>>() {
            handlers
                .get(topic.as_ref())
                .ok_or("handler not found")
                .and_then(|x| {
                    let req = Request::new(arg);
                    x.handler.call(req);
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
        bus.subscribe("topic0".to_string(), |x: i32| {
            println!("closure accept i32")
        });
        bus.subscribe("topic1".to_string(), |x: i32, y: i32| {
            println!("closure accept i32, i32")
        });
        bus.subscribe("topic2".to_string(), |x: i32, y: &str, z: i32| {
            println!("closure accept i32, &str, i32")
        });
        bus.subscribe("topic3".to_string(), || println!("closure have no param"));

        bus.publish("topic1", (1i32, 2i32));
        bus.publish("topic2", (1i32, "100", 2i32));
        bus.publish("topic3", ());
    }
}
