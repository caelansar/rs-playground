use std::borrow::Cow;

#[derive(Debug, PartialEq)]
enum Value {
    String(String),
    Number(i64),
}

impl Value {
    fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
        }
    }

    fn to_str(&self) -> Cow<'_, str> {
        match self {
            Value::String(s) => Cow::Borrowed(s),
            Value::Number(n) => Cow::Owned(n.to_string()),
        }
    }
}

trait TryConvertValue<'a>: Sized {
    type Output;
    fn try_convert_value(value: &'a Value) -> Option<Self::Output>;
}

fn convert<'a, T: TryConvertValue<'a>>(value: &'a Value) -> Option<T::Output> {
    T::try_convert_value(value)
}

impl<'a> TryConvertValue<'a> for i64 {
    type Output = i64;
    fn try_convert_value(value: &'a Value) -> Option<Self::Output> {
        match value {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }
}

impl<'a> TryConvertValue<'a> for String {
    type Output = String;
    fn try_convert_value(value: &'a Value) -> Option<Self::Output> {
        match value {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
        }
    }
}

impl<'a> TryConvertValue<'a> for &str {
    type Output = &'a str;
    fn try_convert_value(value: &'a Value) -> Option<Self::Output> {
        match value {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
}

struct ArgCallback(Box<dyn Fn(&Value) -> Value + Sync + Send + 'static>);

impl ArgCallback {
    pub fn new<F, Arg>(f: F) -> ArgCallback
    where
        Arg: for<'a> TryConvertValue<'a>,
        F: CallbackTrait<Arg> + for<'a> CallbackTrait<<Arg as TryConvertValue<'a>>::Output>,
    {
        ArgCallback(Box::new(move |arg| -> Value {
            f.invoke(Arg::try_convert_value(arg).unwrap())
        }))
    }

    pub fn invoke(&self, arg: &Value) -> Value {
        (self.0)(arg)
    }
}

trait CallbackTrait<Arg>: Send + Sync + 'static {
    fn invoke(&self, args: Arg) -> Value;
}

impl<Func, Arg> CallbackTrait<Arg> for Func
where
    Func: Fn(Arg) -> Value + Send + Sync + 'static,
    Arg: for<'a> TryConvertValue<'a>,
{
    fn invoke(&self, arg: Arg) -> Value {
        (self)(arg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callback() {
        let to_upper =
            ArgCallback::new(for<'a> |x: &'a str| -> Value { Value::String(x.to_uppercase()) });
        let v = to_upper.invoke(&Value::String("cae".to_string()));
        assert_eq!(v, Value::String("CAE".into()));

        let plus_1 = ArgCallback::new(|x: i64| Value::Number(x + 1));
        let v = plus_1.invoke(&Value::Number(1));
        assert_eq!(v, Value::Number(2));
    }
}
