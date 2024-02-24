#[allow(unused_imports)]
#[allow(dead_code)]
mod cli;
use clap::Parser;

use serde_json::Value;
use std::error::Error;
use std::path::PathBuf;
use std::{collections::HashMap, fs, io, str};

// type Dictionary<T> = HashMap<<T>, Bencode>;

// struct InfoDict {
//     info: HashMap<String, Bencode>,
// }

// // // // // // // //
// InfoDict
// // // // // // // //

#[derive(Debug)]
struct InfoDict(HashMap<String, Bencode>);

trait Encode {
    fn bencode(&self) -> Option<String>;
}

impl Encode for InfoDict {
    fn bencode(&self) -> Option<String> {
        println!("impl Encode for InfoDict -> bencode");
        None
    }
}

impl<'a> IntoIterator for &'a InfoDict {
    type Item = (&'a String, &'a Bencode);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Bencode>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<(String, Bencode)> for InfoDict {
    fn from_iter<I: IntoIterator<Item = (String, Bencode)>>(iter: I) -> Self {
        let mut map = HashMap::new();
        for (key, value) in iter {
            map.insert(key, value);
        }
        InfoDict(map)
    }
}

impl InfoDict {
    fn get(&self, key: &str) -> Option<&Bencode> {
        self.0.get(key)
    }
}

// // // // // // // //
// BENCODE
// // // // // // // //

impl Encode for Bencode {
    fn bencode(&self) -> Option<String> {
        match self {
            Self::Dictionary(info_dict) => info_dict.bencode(),
            typed => unimplemented!("bencode only for InfoDict "),
        }
    }
}

#[derive(Debug)]
enum Bencode {
    String(Vec<u8>),
    // String(&'static [u8]),
    Integer(isize),
    List(Vec<Bencode>),
    Dictionary(InfoDict),
}

impl Bencode {
    fn to_json(&self) -> serde_json::Value {
        match self {
            Bencode::String(s) => serde_json::Value::String(str::from_utf8(s).unwrap().to_owned()),
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

    fn length(&self) -> Option<isize> {
        get_info_length(self)
    }
    fn announce(&self) -> Option<String> {
        get_info_announce(self)
    }

    fn info_hash(&self) -> Option<String> {
        get_info_hash(self)
    }
}

fn decode_bencoded_value(encoded_value: &[u8]) -> (Bencode, &[u8]) {
    if encoded_value.first().unwrap().is_ascii_digit() {
        // Example: "5:hello" -> "hello"
        let colon_index = encoded_value.iter().position(|&x| x == b':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = str::from_utf8(number_string)
            .unwrap()
            .parse::<i64>()
            .unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        return (
            Bencode::String(string.to_vec()),
            &encoded_value[colon_index + 1 + number as usize..],
        );
    };

    // i-52e => -52
    if encoded_value.starts_with(&[b'i']) {
        let end_index = encoded_value.iter().position(|&x| x == b'e').unwrap();
        let number_value = &encoded_value[1..end_index];
        return (
            Bencode::Integer(
                str::from_utf8(number_value)
                    .unwrap()
                    .parse::<isize>()
                    .unwrap(),
            ),
            &encoded_value[end_index + 1..],
        );
    };

    // l5:helloi52ee => [“hello”,52]
    if encoded_value.starts_with(&[b'l']) {
        let mut list: Vec<Bencode> = Vec::new();
        let mut remaining = &encoded_value[1..];
        while !remaining.starts_with(&[b'e']) {
            // remaining.chars().next().unwrap() != 'e' {
            let (decoded_value, new_remaining) = decode_bencoded_value(remaining);
            list.push(decoded_value);
            remaining = new_remaining;
        }

        return (Bencode::List(list), &remaining[1..]);
    };

    // d3:foo3:bar5:helloi52ee => {"hello": 52, "foo":"bar"}
    if encoded_value.starts_with(&[b'd']) {
        let mut list: Vec<(String, Bencode)> = Vec::new();
        let mut remaining = &encoded_value[1..];
        while !remaining.starts_with(&[b'e']) {
            // decode key
            let decoded: (Bencode, &[u8]) = decode_bencoded_value(remaining);
            if let (Bencode::String(decoded_key), new_remaining) = decoded {
                // decode value
                let (decoded_value, new_remaining) = decode_bencoded_value(new_remaining);
                list.push((String::from_utf8(decoded_key).unwrap(), decoded_value));
                remaining = new_remaining;
            }
        }

        (
            Bencode::Dictionary(list.into_iter().collect()),
            &remaining[1..],
        )
    } else {
        panic!("Unhandled value, {:?}", encoded_value);
    }
}

fn read_file_to_vec(filename: &PathBuf) -> io::Result<Vec<u8>> {
    fs::read(filename)
}

fn get_info_length(decoded: &Bencode) -> Option<isize> {
    if let Bencode::Dictionary(ref outer_dict) = decoded {
        if let Some(Bencode::Dictionary(info)) = outer_dict.get("info") {
            if let Some(Bencode::Integer(length)) = info.get("length") {
                return Some(*length);
            }
        }
    }
    None
}

fn get_info_announce(decoded: &Bencode) -> Option<String> {
    if let Bencode::Dictionary(ref outer_dict) = decoded {
        if let Some(Bencode::String(announce_url)) = outer_dict.get("announce") {
            return Some(String::from_utf8(announce_url.clone()).unwrap());
        }
    }
    None
}

fn get_info_hash(decoded: &Bencode) -> Option<String> {
    if let Bencode::Dictionary(ref outer_dict) = decoded {
        if let Some(bencode_info_dict) = outer_dict.get("info") {
            return calculate_hash(bencode_info_dict.bencode());
        }
    }
    None
}

fn calculate_hash(input: Option<String>) -> Option<String> {
    Some("hashvalue".to_owned())
}

// // // // // // // //
// main
// // // // // // // //

fn main() -> Result<(), Box<dyn Error>> {
    let cla = cli::Cli::parse();
    match cla.commands {
        cli::Commands::Decode { bencoded_value } => {
            let bytes_slice = bencoded_value.as_bytes();
            let (decoded, _) = decode_bencoded_value(&bytes_slice);
            println!("{}", decoded.to_json().to_owned());
        }
        cli::Commands::Info { torrent_file } => {
            let file_contents = read_file_to_vec(&torrent_file).unwrap();
            let bytes_slice = file_contents.as_slice();
            let (decoded, _) = decode_bencoded_value(&bytes_slice);
            if let Some(url) = decoded.announce() {
                println!("Tracker URL: {}", url);
            }
            if let Some(length) = decoded.length() {
                println!("Length: {}", length);
            }

            if let Some(info_hash) = decoded.info_hash() {
                println!("Info Hash: {}", info_hash);
            }
        }
        command => {
            println!("Command: {:?} not implemented!", command);
        }
    }

    Ok(())
}
