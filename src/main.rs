extern crate regex;
use regex::Regex;


fn main() {
    println!("Hello, world!");
}


pub fn extract_id(log_line: &str) -> Option<i32> {
	let re = Regex::new(r"\[([0-9]+)\]").unwrap();
	let cap = re.captures(log_line);

	let m = cap.unwrap().at(1);
	println!("{}", m.unwrap());

	let result = m.unwrap().parse::<i32>();

	match result {
		Ok(i) => {
			return Some(i)
		}
		Err(_e) => {
			return None
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_extract() {
		let result: Option<i32> = extract_id("Some [123] log line");
		assert_eq!(result, Some(123))
	}

	#[test]
	fn test_extract2() {
		let result: Option<i32> = extract_id("Some [9] log line");
		assert_eq!(result, Some(9))
	}

	#[test]
	#[should_panic]
	fn test_no_id() {
		let result: Option<i32> = extract_id("Some log line without an ID");
	}

}
