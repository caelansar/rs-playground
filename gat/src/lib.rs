mod family;
mod lending_iter;
mod mapper;

use family::*;
use lending_iter::*;
use mapper::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn uni_map<T, U, M, F>(mapper: M, f: F) -> M::Result<U>
    where
        M: Mapper<Item = T>,
        F: FnMut(T) -> U,
    {
        mapper.map(f)
    }

    struct MyStruct<P: PointerFamily> {
        pointer: P::Pointer<String>,
    }

    #[test]
    fn map_option() {
        let v = Some(1);

        let v1 = uni_map(v, |x| x + 1);
        assert_eq!(Some(2), v1)
    }

    #[test]
    fn map_vec() {
        let v = vec![1, 2, 3];

        let v1 = uni_map(v, |x| x + 1);
        assert_eq!(vec![2, 3, 4], v1)
    }

    #[test]
    fn map_result() {
        let v: Result<i32, &str> = Ok(1);

        let v1 = uni_map(v, |x| x + 1);
        assert_eq!(Ok(2), v1)
    }

    #[test]
    fn test_family() {
        fn is_send<T: Send>(_: T) {}
        let s: MyStruct<RcFamily> = MyStruct {
            pointer: RcFamily::new("aa".to_string()),
        };
        // Rc<String>` cannot be sent between threads safely
        // is_send(s);

        let s: MyStruct<ArcFamily> = MyStruct {
            pointer: ArcFamily::new("aa".to_string()),
        };
        is_send(s);
    }

    #[test]
    fn test_lending_trait() {
        let mut data = [1, 2, 3, 4, 5, 6];
        let mut win = WindowsMut::new(&mut data, 0, 3);
        loop {
            match win.next() {
                Some(data) => println!("{:?}", data),
                None => break,
            }
        }
    }
}
