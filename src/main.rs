#[allow(unused_imports)]
// Available if you need it!
use serde_bencode;
use serde_json;
use std::env;

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    match encoded_value.chars().next() {
        Some(c) if c.is_digit(10) => {
            let colon_index = encoded_value.find(':').unwrap();
            let number_string = &encoded_value[..colon_index];
            let number = number_string.parse::<i64>().unwrap();
            let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
            serde_json::Value::String(string.to_string())
        }
        Some(c) if c.is_alphabetic() => {
            let int_type = encoded_value.find("i").unwrap();
            let number_string = &encoded_value[int_type + 1..encoded_value.len() - 1];
            let number = number_string.parse::<i64>().unwrap();

            serde_json::Value::from(number)
        }
        Some(_) => {
            panic!("Unhandled encoded value: {}", encoded_value)
        }
        None => panic!("Unhandled encoded value: {}", encoded_value),
    }
}
// If encoded_value starts with a digit, it's a number
// if encoded_value.chars().next().unwrap().is_digit(10) {
//     // Example: "5:hello" -> "hello"
//     let colon_index = encoded_value.find(':').unwrap();
//     let number_string = &encoded_value[..colon_index];
//     let number = number_string.parse::<i64>().unwrap();
//     let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
//     return serde_json::Value::String(string.to_string());
// } else if encoded_value.chars().next().unwrap().is_alphabetic() {
//     let int_type = encoded_value.find("i").unwrap();
//     let number_string = &encoded_value[int_type + 1..encoded_value.len() - 1];
//     let number = number_string.parse::<i64>().unwrap();

//     return serde_json::Value::from(number);
// } else {
//     panic!("Unhandled encoded value: {}", encoded_value)
// }
// }

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        eprintln!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else {
        eprintln!("unknown command: {}", args[1])
    }
}
