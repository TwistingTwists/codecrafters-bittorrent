#[allow(unused_imports)]
// Available if you need it!
use serde_json::Value;
use std::{collections::HashMap, env};

enum Bencode {
    String(String),
    Integer(isize),
    List(Vec<Bencode>),
    Dictionary(HashMap<String, Bencode>),
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
            Bencode::Dictionary(d) => {
                let mut map = serde_json::Map::new();
                for (key, value) in d.into_iter() {
                    map.insert(key.clone(), value.to_json());
                }
                Value::Object(map)
            }
        }
    }
}

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

        return (Bencode::List(list), &remaining[1..]);
    };

    // d3:foo3:bar5:helloi52ee => {"hello": 52, "foo":"bar"}
    if encoded_value.chars().nth(0).unwrap() == 'd' {
        let mut list: Vec<(String, Bencode)> = Vec::new();
        let mut remaining = &encoded_value[1..];
        while remaining.chars().next().unwrap() != 'e' {
            // decode key
            let decoded: (Bencode, &str) = decode_bencoded_value(remaining);
            if let (Bencode::String(decoded_key), new_remaining) = decoded {
                // decode value
                let (decoded_value, new_remaining) = decode_bencoded_value(new_remaining);
                list.push((decoded_key, decoded_value));
                remaining = new_remaining;
            }
        }

        (
            Bencode::Dictionary(list.into_iter().collect()),
            &remaining[1..],
        )
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

#[cfg(test)]
mod tests {
    use super::*; // Import the necessary items from the outer module
    use serde_json::json;

    #[test]
    fn test_bencode_to_json_conversion() {
        // Construct the Bencode dictionary as in the example
        let bencode_example = Bencode::Dictionary(
            vec![
                ("key1".to_owned(), Bencode::String("value1".to_owned())),
                ("key2".to_owned(), Bencode::Integer(42)),
            ]
            .into_iter()
            .collect(),
        );

        // Convert the Bencode instance into a serde_json::Value
        let json_value: serde_json::Value = bencode_example.to_json();

        // Expected JSON output
        let expected_json = json!({
            "key1": "value1",
            "key2": 42
        });

        // Assert that the conversion matches the expected JSON output
        assert_eq!(json_value, expected_json);
    }
}
