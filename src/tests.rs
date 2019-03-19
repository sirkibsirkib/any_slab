use crate::*;

use rand::Rng;
use hashbrown::HashMap;
use rand::seq::SliceRandom;

enum TestCase {
	InsertU32,
	InsertF32,
	InsertI128,
	RmU32,
	RmF32,
	RmI128,
}

#[test]
fn idk() {
	let mut a = AnySlab::default();
	let mut rng = rand::thread_rng();
	let mut u32s: HashMap<usize,u32> = HashMap::default();
	let mut f32s: HashMap<usize,f32> = HashMap::default();
	let mut i128s: HashMap<usize,i128> = HashMap::default();

	for _ in 0..4000 {
		use TestCase::*;
		let test = if a.len() < 100 {
			[InsertU32, InsertF32, InsertI128].choose(&mut rng)
		} else {
			[RmU32, RmF32, RmI128].choose(&mut rng)
		}.unwrap();
		match test {
			TestCase::InsertU32 => {
				let x: u32 = rng.gen();
				let k = a.insert(x);
				println!("put u32 k:{} v:{}", k, x);
				u32s.insert(k, x);
			},
			TestCase::InsertF32 => {
				let x: f32= rng.gen();
				let k = a.insert(x);
				println!("put f32 k:{} v:{}", k, x);
				f32s.insert(k, x);
			},
			TestCase::InsertI128 => {
				let x: i128 = rng.gen();
				let k = a.insert(x);
				println!("put i128 k:{} v:{}", k, x);
				i128s.insert(k, x);
			},
			TestCase::RmU32 => {
				if let Some(&k) = u32s.keys().next() {
					let v = u32s.remove(&k).unwrap();
					println!("rm u32 k:{} v:{}", k, v);
					let got = a.remove(k).expect("BAD u32");
					assert_eq!(v, got);
				}
			},
			TestCase::RmF32 => {
				if let Some(&k) = f32s.keys().next() {
					let v = f32s.remove(&k).unwrap();
					println!("rm f32 k:{} v:{}", k, v);
					let got = a.remove(k).expect("BAD f32");
					assert_eq!(v, got);
				}
			},
			TestCase::RmI128 => {
				if let Some(&k) = i128s.keys().next() {
					let v = i128s.remove(&k).unwrap();
					println!("rm i128 k:{} v:{}", k, v);
					let got = a.remove(k).expect("BAD i128");
					assert_eq!(v, got);
				}
			},
		}
	}
}
