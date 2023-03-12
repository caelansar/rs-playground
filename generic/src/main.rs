#![allow(unused)]
#![feature(min_specialization)]

use specialization::Test;

mod no_generic {
    use std::collections::HashMap;

    pub struct PasswordManager {
        master_pass: String,
        passwords: HashMap<String, String>,
    }

    impl PasswordManager {
        pub fn new(master_pass: String) -> Self {
            PasswordManager {
                master_pass,
                passwords: Default::default(),
            }
        }
        pub fn unlock(&mut self, master_pass: String) {}

        pub fn lock(&mut self) {}

        pub fn list_passwords(&self) -> &HashMap<String, String> {
            &self.passwords
        }

        pub fn add_password(&mut self, username: String, password: String) {
            self.passwords.insert(username, password);
        }

        pub fn encryption(&self) -> String {
            String::default()
        }

        pub fn version(&self) -> String {
            String::default()
        }
    }
}

mod generic {
    use std::collections::HashMap;
    use std::marker::PhantomData;

    pub struct Locked;
    pub struct Unlocked;

    pub struct PasswordManager<State = Locked> {
        master_pass: String,
        passwords: HashMap<String, String>,
        state: PhantomData<State>,
    }

    impl PasswordManager<Locked> {
        pub fn unlock(self, master_pass: String) -> PasswordManager<Unlocked> {
            PasswordManager {
                master_pass: self.master_pass,
                passwords: self.passwords,
                state: PhantomData,
            }
        }
    }

    impl PasswordManager<Unlocked> {
        pub fn lock(self) -> PasswordManager<Locked> {
            PasswordManager {
                master_pass: self.master_pass,
                passwords: self.passwords,
                state: PhantomData,
            }
        }

        pub fn list_passwords(&self) -> &HashMap<String, String> {
            &self.passwords
        }

        pub fn add_password(&mut self, username: String, password: String) {
            self.passwords.insert(username, password);
        }
    }

    impl<State> PasswordManager<State> {
        pub fn encryption(&self) -> String {
            String::default()
        }

        pub fn version(&self) -> String {
            String::default()
        }
    }

    impl PasswordManager {
        pub fn new(master_pass: String) -> Self {
            PasswordManager {
                master_pass,
                passwords: Default::default(),
                state: PhantomData,
            }
        }
    }
}

mod specialization {
    use std::fmt::Debug;

    pub trait Test {
        fn test(&self);
    }

    impl<T: Debug> Test for T {
        default fn test(&self) {
            println!("{:?} impl Test trait", self);
        }
    }

    pub struct A;

    impl Test for A {
        fn test(&self) {
            println!("A impl Test trait");
        }
    }
}

fn main() {
    let manager = generic::PasswordManager::new("password".to_owned());
    let mut manager = manager.unlock("password".to_owned());
    manager.list_passwords();
    manager.add_password("u1".to_string(), "p1".to_string());
    manager.lock();

    let mut manager = no_generic::PasswordManager::new("password".to_owned());
    manager.unlock("password".to_owned());
    manager.list_passwords();
    manager.lock();
    manager.list_passwords(); // misuse
    manager.lock(); // misuse

    let a = specialization::A;
    a.test();

    0.test();
}
