#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
mod cli;
use clap::Parser;
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
// use std::ascii::AsciiExt;
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
    fn bencode(&self) -> Vec<u8>;
}

impl Encode for InfoDict {
    fn bencode(&self) -> Vec<u8> {
        let InfoDict(hashmap) = self;
        let hash_map_as_vec_u8 = hashmap_bencode(hashmap);

        // println!("{:#?} infodict - vec", hash_map_as_vec_u8);
        hash_map_as_vec_u8
        // Some(hash_map_as_str)
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
    fn bencode(&self) -> Vec<u8> {
        match self {
            Self::String(vec_u8) => {
                let return_val = string_bencode(vec_u8);
                // println!("{:#?} string", return_val);
                return_val
            }
            Self::Integer(isize_int) => {
                let return_val = integer_bencode(isize_int);
                // println!("{:#?} isize", return_val);
                return_val
            }
            Self::List(vec_bencode) => {
                let return_val = list_bencode(vec_bencode);
                // println!("{:#?} list", return_val);
                return_val
            }
            Self::Dictionary(info_dict) => {
                // println!("{:#?} info_dict", info_dict);

                info_dict.bencode()
            } // typed => unimplemented!("bencode only for InfoDict "),
        }
    }
}

fn string_bencode(vec_u8: &Vec<u8>) -> Vec<u8> {
    let mut new_vec_u8: Vec<u8> = Vec::new();
    let length: u8 = vec_u8
        .len()
        .try_into() // Convert usize to u8
        .expect("Length cannot exceed 255");

    new_vec_u8.push(length);

    new_vec_u8.push(b':');
    new_vec_u8.extend(vec_u8);
    // println!("{:#?} new_vec_u8 in string bencode", new_vec_u8);
    new_vec_u8
    // String::from_utf8(new_vec_u8).ok()
}

fn integer_bencode(isize_int: &isize) -> Vec<u8> {
    let mut new_vec_u8: Vec<u8> = Vec::new();

    new_vec_u8.push(b'i');
    // -52 -> "-52"
    let number_string = isize_int.to_string();
    // isize may have optional "-" in front
    let bytes: Vec<u8> = if let Some('-') = number_string.chars().next() {
        new_vec_u8.push(b'-');
        number_string[1..]
            .chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect()
    } else {
        number_string
            .chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect()
    };
    new_vec_u8.extend(bytes);
    new_vec_u8.push(b'e');
    new_vec_u8
}

fn list_bencode(vec_bencode: &Vec<Bencode>) -> Vec<u8> {
    // Use map() to apply .bencode() on each element
    let bencoded_elements: Vec<u8> = vec_bencode
        .iter()
        .flat_map(|bencode| bencode.bencode())
        .collect();
    bencoded_elements
}

fn hashmap_bencode(hashmap: &HashMap<String, Bencode>) -> Vec<u8> {
    // use sorted keys in hashmap
    // Convert the HashMap to a BTreeMap with sorted keys
    // let btreemap: BTreeMap<String, Bencode> = hashmap.into_iter().collect();

    // Get the keys from the hashmap
    let mut sorted_keys: Vec<String> = hashmap.keys().cloned().collect();

    // Sort the keys
    sorted_keys.sort();

    println!("{:?}", sorted_keys);

    let mut bencoded_pairs: Vec<u8> = Vec::new();
    bencoded_pairs.push(b'd');

    // Iterate over the sorted keys and access the values from the hashmap
    for bencoded_key in sorted_keys {
        if let Some(value) = hashmap.get(&bencoded_key) {
            let bencoded_value = value.bencode();
            println!("key: {:?}", bencoded_key);
            println!(
                "value: {:?}",
                bencoded_value // String::from_utf8(bencoded_value.clone()).ok()
            );

            bencoded_pairs.extend(bencoded_key.as_bytes());
            bencoded_pairs.extend(bencoded_value);
        }
    }
    bencoded_pairs.push(b'e');
    bencoded_pairs
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
        if let Some(Bencode::Dictionary(bencode_info_dict)) = outer_dict.get("info") {
            return calculate_hash(bencode_info_dict.bencode());
        }
    }
    None
}

fn calculate_hash(input: Vec<u8>) -> Option<String> {
    // Create a SHA-1 hasher
    let mut hasher = Sha1::new();

    // println!("{:?}", input);
    // Update the hasher with the bytes of the dictionary
    hasher.update(input);

    // Calculate the final hash
    let result = hasher.finalize();
    let hex_string = format!("{:02x}", result);

    // println!("calculate hash = {:?} ", hex_string);
    Some(hex_string)
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
    }

    Ok(())
}
