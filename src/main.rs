#[allow(unused_imports)]
// Available if you need it!
use serde_bencode;
use serde_json;
use std::env;

enum Bencode {
    String(String),
    Integer(isize),
    List(Vec<Bencode>),
}

impl Bencode {
    fn to_json(&self) -> serde_json::Value {
        match self {
            Bencode::String(s) => serde_json::Value::String(s.clone()),
            Bencode::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Bencode::List(l) => {
                let mut result = serde_json::Value::Array(Vec::new());
                for bencode in l {
                    result.as_array_mut().unwrap().push(bencode.to_json());
                }
                result
            }
        }
    }
}

// #[allow(dead_code)]
// fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
//     match encoded_value.chars().next() {
//         Some(c) if c.is_digit(10) => {
//             let colon_index = encoded_value.find(':').unwrap();
//             let number_string = &encoded_value[..colon_index];
//             let number = number_string.parse::<i64>().unwrap();
//             let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
//             serde_json::Value::String(string.to_string())
//         }
//         Some(c) if c.is_alphabetic() => match c {
//             'i' => {
//                 let int_type = encoded_value.find("i").unwrap();
//                 let number_string = &encoded_value[int_type + 1..encoded_value.len() - 1];
//                 let number = number_string.parse::<i64>().unwrap();

//                 serde_json::Value::from(number)
//             }
//             // l5:hellowi52ee
//             'l' => {
//                 let colon_index = encoded_value.find(':').unwrap();
//                 let string = &encoded_value[colon_index + 1..];

//                 decode_bencoded_value(encoded_value)
//             }
//             _ => {
//                 panic!("Unhandled encoded value: {}", encoded_value)
//             }
//         },
//         Some(_) => {
//             panic!("Unhandled encoded value: {}", encoded_value)
//         }
//         None => panic!("Unhandled encoded value: {}", encoded_value),
//     }
// }

fn decode_bencoded_value(encoded_value: &str) -> (Bencode, &str) {
    if encoded_value.chars().next().unwrap().is_digit(10) {
        // Example: "5:hello" -> "hello"
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<i64>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        return (
            Bencode::String(String::from(string)),
            &encoded_value[colon_index + 1 + number as usize..],
        );
    };

    if encoded_value.chars().nth(0).unwrap() == 'i' {
        let end_index = encoded_value.find('e').unwrap();
        let number_value = &encoded_value[1..end_index];
        return (
            Bencode::Integer(number_value.parse::<isize>().unwrap()),
            &encoded_value[end_index + 1..],
        );
    };

    if encoded_value.chars().nth(0).unwrap() == 'l' {
        let mut list: Vec<Bencode> = Vec::new();
        let mut remaining = &encoded_value[1..];
        while remaining.chars().next().unwrap() != 'e' {
            let (decoded_value, new_remaining) = decode_bencoded_value(remaining);
            list.push(decoded_value);
            remaining = new_remaining;
        }

        (Bencode::List(list), &remaining[1..])
    } else {
        panic!("Unhandled value, {}", encoded_value);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        eprintln!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let (decoded_value, _) = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_json().to_string());
    } else {
        eprintln!("unknown command: {}", args[1])
    }
}
