use std::env;
use std::rc::Rc;
use std::cell::{RefCell};
use std::collections::{VecDeque, HashMap};
use std::fmt;

pub type RcValue = Rc<RefCell<String>>;

#[derive(Clone)]
pub enum ItemValue {
    Single(RcValue),
    Multi(VecDeque<RcValue>)
}

impl fmt::Display for ItemValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ItemValue::Single(v) => {
                v.borrow().fmt(f)
            },
            ItemValue::Multi(v) => {
                let mut r = String::new();
                for item in v {
                    r.push_str(&*item.borrow());
                    r.push(' ');
                }
                write!(f, "{}", r)
            }
        }
    }
}

struct Item {
    value: ItemValue,
    desc: String,
    is: bool,
    value_len: isize
}

pub struct Flag {
    help: String,
    keys: HashMap<String, Item>,
    is_warning: bool
}

pub struct Value {
    pub v: ItemValue
}

impl Value {
    fn new(v: ItemValue) -> Self {
        Self {
            v: v
        }
    }
}

enum ReadStatus {
    Processing,
    Finish
}

struct Reader {
    value: ItemValue,
    value_len: isize,
    index: usize
}

impl Reader {
    fn process(&mut self, arg: String) -> ReadStatus {
        match &mut self.value {
            ItemValue::Single(v) => {
                *v.borrow_mut() = arg;
            },
            ItemValue::Multi(v) => {
                if self.index < v.len() {
                    *v[self.index].borrow_mut() = arg;
                } else {
                    v.push_back(Rc::new(RefCell::new(arg)));
                }
            }
        }
        self.index += 1;
        if self.value_len < 0 {
            ReadStatus::Processing
        } else {
            if self.index == self.value_len as usize {
                ReadStatus::Finish
            } else {
                ReadStatus::Processing
            }
        }
    }

    fn next_key(&self) -> ReadStatus {
        if self.value_len < 0 {
            ReadStatus::Finish
        } else {
            if self.index != self.value_len as usize {
                ReadStatus::Processing
            } else {
                ReadStatus::Finish
            }
        }
    }

    fn new(value: ItemValue, value_len: isize) -> Self {
        Self {
            value: value,
            value_len: value_len,
            index: 0
        }
    }
}

pub trait ToItem {
    fn to_item(self) -> ItemValue;
}

impl ToItem for VecDeque<String> {
    fn to_item(mut self) -> ItemValue {
        let mut v = VecDeque::with_capacity(self.len());
        while self.len() > 0 {
            v.push_back(RcValue::new(RefCell::new(self.pop_front().unwrap())));
        }
        ItemValue::Multi(v)
    }
}

impl ToItem for String {
    fn to_item(self) -> ItemValue {
        ItemValue::Single(RcValue::new(RefCell::new(self)))
    }
}

impl ToItem for u32 {
    fn to_item(self) -> ItemValue {
        ItemValue::Single(RcValue::new(RefCell::new(self.to_string())))
    }
}

fn panic<T: std::fmt::Display>(msg: T) {
    println!("{}", msg);
    std::process::exit(0);
}

impl Flag {
    fn register<T: ToItem>(&mut self, key: String, default: T
        , value_len: isize) -> Value {
        self.register_with_desc(key, default, String::from(""), value_len)
    }

    fn register_with_desc<T: ToItem>(&mut self, key: String, default: T
        , desc: String, value_len: isize) -> Value {
        let r = default.to_item();
        self.keys.insert(key.to_string(), Item{
            value: r.clone(),
            desc: desc,
            is: false,
            value_len: value_len
        });
        Value::new(r)
    }

    pub fn reg_string(&mut self, key: String, default: String, desc: String) -> Value {
        self.register_with_desc(key
            , default, desc, 1)
    }

    pub fn reg_u32(&mut self, key: String, default: u32, desc: String) -> Value {
        self.register_with_desc(key
            , default, desc, 1)
    }

    pub fn reg_fixed_str_vec(&mut self, key: String, default: VecDeque<String>
        , desc: String) -> Value {
        let len = default.len();
        self.register_with_desc(key
            , default, desc, len as isize)
    }

    pub fn reg_lengthen_str_vec(&mut self, key: String, default: VecDeque<String>
        , desc: String) -> Value {
        self.register_with_desc(key
            , default, desc, -1)
    }

    pub fn has(&self, key: &str) -> bool {
        let v = match self.keys.get(key) {
            Some(v) => v,
            None => {
                return false;
            }
        };
        v.is
    }

    pub fn parse(&mut self) {
        let args = env::args();
        let mut reader: Option<Reader> = None;
        let mut read_status = ReadStatus::Finish;
        for (i, arg) in args.enumerate() {
            if arg == self.help {
                self.print_help();
                self.exit();
            }
            match self.keys.get(&arg) {
                Some(item) => {
                    if let Some(r) = &reader {
                        read_status = r.next_key();
                    };
                    if let ReadStatus::Processing = &read_status {
                        panic(format!(
                                "the parameters before the {} parameter are not matched"
                                , arg));
                    }
                    reader = Some(Reader::new(item.value.clone()
                            , item.value_len));
                    continue;
                },
                None => {
                }
            }
            match &mut reader {
                Some(r) => {
                    read_status = r.process(arg);
                    if let ReadStatus::Finish = &read_status {
                        reader = None;
                    }
                },
                None => {
                }
            }
        }
    }

    fn warning<T: std::fmt::Display>(&self, msg: T) {
        if self.is_warning {
            println!("[Warning] {}", msg);
        }
    }

    fn print_help(&self) {
        println!("help:");
        for (key, value) in self.keys.iter() {
            println!("\t{}\n\t\tdefault: {}\n\t\tdesc: {}", key, value.value, &value.desc);
        }
    }

    fn exit(&self) {
        std::process::exit(0);
    }

    pub fn set_help(&mut self, help: String) {
        *&mut self.help = help;
    }

    pub fn set_warning(&mut self) {
        self.is_warning = true;
    }

    pub fn set_nowarning(&mut self) {
        self.is_warning = false;
    }

    pub fn new() -> Self {
        Self {
            help: "--help".to_string(),
            keys: HashMap::new(),
            is_warning: true
        }
    }
}

#[macro_export]
macro_rules! read {
    ($v:ident, $typ:ident) => {
        match $v.v {
            ItemValue::Single(v) => {
                match v.borrow().parse::<$typ>() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("[ERROR] file: {}, line: {}, var \"{}\": to {} error"
                            , file!(), line!(), stringify!($v), stringify!($typ));
                        std::process::exit(0);
                    }
                }
            },
            ItemValue::Multi(_) => {
                println!("[ERROR] value is single");
                std::process::exit(0);
            }
        }
    }
}

#[macro_export]
macro_rules! read_i32 {
    ($v:ident) => {
        read!($v, i32)
    }
}

#[macro_export]
macro_rules! read_u32 {
    ($v:ident) => {
        read!($v, u32)
    }
}

#[macro_export]
macro_rules! read_string {
    ($v:expr) => {
        &*match $v.v {
            ItemValue::Single(v) => v,
            ItemValue::Multi(_) => {
                println!("[ERROR] value is single");
                std::process::exit(0);
            }
        }.borrow()
    }
}

#[macro_export]
macro_rules! read_string_item {
    ($v:expr) => {
        $v.borrow()
    }
}

#[macro_export]
macro_rules! read_item {
    ($v:ident, $typ:ident) => {
        match $v.borrow().parse::<$typ>() {
            Ok(v) => v,
            Err(_) => {
                println!("[ERROR] file: {}, line: {}, var \"{}\": to {} error"
                    , file!(), line!(), stringify!($v), stringify!($typ));
                std::process::exit(0);
            }
        }
    }
}

#[macro_export]
macro_rules! read_vector {
    ($v:ident) => {
        &match $v.v {
            ItemValue::Multi(v) => v,
            ItemValue::Single(_) => {
                println!("[ERROR] value is single");
                std::process::exit(0);
            }
        }
    }
}

#[macro_export]
macro_rules! vecdeque {
    ($($value:expr),*) => (
        {
            let mut v = std::collections::VecDeque::new();
            $(v.push_back($value);)*
            v
        }
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn flag_test() {
        let mut flag = Flag::new();
        let host = flag.reg_string(String::from("-h"), String::from("localhost")
            , String::from("host"));
        let port = flag.reg_u32(String::from("-p"), 80
            , String::from("port"));
        let address = flag.reg_lengthen_str_vec(String::from("-address")
            , vecdeque!["a".to_string(), "b".to_string(), "c".to_string()]
            , String::from("address"));
        let packages = flag.reg_fixed_str_vec(String::from("-packages")
            , vecdeque!["libmath".to_string(), "../third".to_string()]
            , String::from("packages"));
        flag.parse();
        println!("{}", read_string!(host));
        println!("{}", read_i32!(port));
        for item in read_vector!(address) {
            println!("{}", read_string_item!(item));
        }
        for item in read_vector!(packages) {
            println!("{}", read_string_item!(item));
        }
    }
}

