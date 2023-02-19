fn call(arg: u64, f: impl Fn(u64) -> u64) -> u64 {
    f(arg)
}

fn call_mut(arg: u64, mut f_mut: impl FnMut(u64) -> u64) -> u64 {
    f_mut(arg)
}

fn call_once(arg: u64, f_once: impl FnOnce(u64) -> u64) -> u64 {
    f_once(arg)
}

pub trait Executor {
    fn execute(&self, cmd: &str) -> Result<String, &'static str>;
}

struct BashExecutor {
    env: String,
}

impl Executor for BashExecutor {
    fn execute(&self, cmd: &str) -> Result<String, &'static str> {
        Ok(format!(
            "noop struct execute: env: {}, cmd: {}",
            self.env, cmd
        ))
    }
}

impl<T> Executor for T
where
    T: Fn(&str) -> Result<String, &'static str>,
{
    fn execute(&self, cmd: &str) -> Result<String, &'static str> {
        self(cmd)
    }
}

fn execute(cmd: &str, exec: impl Executor) -> Result<String, &'static str> {
    exec.execute(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closure_works() {
        let v = vec![0u8; 1];
        let v1 = vec![0u8; 2];

        let c = |x: u64| v.len() as u64 * x;

        let c1 = move |x: u64| v1.len() as u64 * x;

        println!("direct call: {}", c(2));
        println!("direct call: {}", c1(2));

        println!("call fn: {}", call(2, c));
        println!("call fn: {}", call(2, &c1));

        println!("call fn_mut: {}", call_mut(2, c));
        println!("call fn_mut: {}", call_mut(2, &c1));

        println!("call fn_once: {}", call_once(2, c));
        println!("call fn_once: {}", call_once(2, c1));

        let name = String::from("cae");
        let vec = vec!["a", "b", "c"];
        let v = &vec[..];
        let data = (1, 2, 3, 4);

        let c = move || {
            println!("data: {:?}", data);
            println!("v: {:?}, name: {:?}", v, name.clone());
        };
        // struct Closure<'a, 'b: 'a> {
        //     data: (i32, i32, i32, i32),
        //     v: &'a [&'b str],
        //     name: String,
        // }
        c();

        // can not access name, since it moved into closure
        // println!("name {}", name); ❌
        //
        // let c = || {
        //     println!("data: {:?}", data);
        //     println!("vec: {:?}, name: {:?}", v, name.clone());
        // };

        // c();

        // println!("name {}", name);
    }

    #[test]
    fn closure_impl_trait() {
        let env = "PATH=/usr/bin".to_string();

        let cmd = "ls";
        let r1 = execute(cmd, BashExecutor { env: env.clone() });
        println!("{:?}", r1);

        let r2 = execute(cmd, |cmd: &str| {
            Ok(format!("noop closure execute: env: {}, cmd: {}", env, cmd))
        });
        println!("{:?}", r2);
    }
}
