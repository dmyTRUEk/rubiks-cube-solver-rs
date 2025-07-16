//! Rubiks Cube Solver by meet-in-the-middle search.

use std::{cmp::{Ordering, Reverse}, collections::BinaryHeap, time::Instant};

use rand::{rng, rngs::ThreadRng, Rng};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};



// const CORES_N: usize = 1;
const CORES_N: usize = 10;



fn main() {
	// loop {
	// println!("{}", "-".repeat(42));
	let time_begin = Instant::now();
	let mut rc = RubiksCube::new();
	let moves = rc.shuffle(11, &mut rng());
	dbg!(&moves, moves.len());
	let solution = rc.solve(&RubiksCube::new());
	dbg!(&solution, solution.len());
	let time_end = Instant::now();
	let elapsed = time_end - time_begin;
	dbg!(elapsed);
	// }
}





//       y y y
//       y y y
//       y y y
// b b b r r r g g g o o o
// b b b r r r g g g o o o
// b b b r r r g g g o o o
//       w w w
//       w w w
//       w w w
//
//            0  1  2
//            3  4  5
//            6  7  8
//  9 10 11  12 13 14  15 16 17  18 19 20
// 21 22 23  24 25 26  27 28 29  30 31 32
// 33 34 35  36 37 38  39 40 41  42 43 44
//           45 46 47
//           48 49 50
//           51 52 53
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RubiksCube {
	pieces: [Color; 9*6]
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Color { Y, B, R, G, O, W }
const ALL_COLORS: [Color; 6] = {use Color::*; [Y, B, R, G, O, W]};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Move { Front, FrontS, Back, BackS, Left, LeftS, Right, RightS, Top, TopS, Bottom, BottomS, X, Xs, Y, Ys, Z, Zs }
const ALL_MOVES: [Move; 18] = {use Move::*; [Front, FrontS, Back, BackS, Left, LeftS, Right, RightS, Top, TopS, Bottom, BottomS, X, Xs, Y, Ys, Z, Zs]};

impl RubiksCube {
	const NEW: [Color; 9*6] = { use Color::*; [
			  Y,Y,Y,
			  Y,Y,Y,
			  Y,Y,Y,
		B,B,B,R,R,R,G,G,G,O,O,O,
		B,B,B,R,R,R,G,G,G,O,O,O,
		B,B,B,R,R,R,G,G,G,O,O,O,
			  W,W,W,
			  W,W,W,
			  W,W,W,
	]};

	fn new() -> Self {
		Self { pieces: Self::NEW }
	}

	fn from(pieces: [Color; 9*6]) -> Self {
		Self { pieces }
	}

	fn new_shuffled(n: u32, rng: &mut ThreadRng) -> Self {
		let mut self_ = Self::new();
		self_.shuffle(n, rng);
		self_
	}

	fn shuffle(&mut self, n: u32, rng: &mut ThreadRng) -> Vec<Move> {
		let mut moves = vec![];
		for _ in 0..n {
			let move_ = self.shuffle_once(rng);
			moves.push(move_);
		}
		moves
	}
	fn shuffle_once(&mut self, rng: &mut ThreadRng) -> Move {
		use Move::*;
		match rng.gen_range(0..=17) {
			0  => { self.front(); Front },
			1  => { self.front_s(); FrontS },
			2  => { self.back(); Back },
			3  => { self.back_s(); BackS },
			4  => { self.left(); Left },
			5  => { self.left_s(); LeftS },
			6  => { self.right(); Right },
			7  => { self.right_s(); RightS },
			8  => { self.top(); Top },
			9  => { self.top_s(); TopS },
			10 => { self.bottom(); Bottom },
			11 => { self.bottom_s(); BottomS },
			12 => { self.x(); X },
			13 => { self.x_s(); Xs },
			14 => { self.y(); Y },
			15 => { self.y_s(); Ys },
			16 => { self.z(); Z },
			17 => { self.z_s(); Zs },
			_ => unreachable!(),
		}
	}

	fn make_move(&mut self, move_: Move) {
		match move_ {
			Move::Front => self.front(),
			Move::FrontS => self.front_s(),
			Move::Back => self.back(),
			Move::BackS => self.back_s(),
			Move::Left => self.left(),
			Move::LeftS => self.left_s(),
			Move::Right => self.right(),
			Move::RightS => self.right_s(),
			Move::Top => self.top(),
			Move::TopS => self.top_s(),
			Move::Bottom => self.bottom(),
			Move::BottomS => self.bottom_s(),
			Move::X => self.x(),
			Move::Xs => self.x_s(),
			Move::Y => self.y(),
			Move::Ys => self.y_s(),
			Move::Z => self.z(),
			Move::Zs => self.z_s(),
		}
	}

	fn solve(&self, other: &RubiksCube) -> Vec<Move> {
		let rc_init: RubiksCube = self.clone();
		let rc_final: RubiksCube = other.clone();
		let mut left_rcs: SortedVec<RubiksCube> = SortedVec::from_array([rc_init.clone()]);
		let mut right_rcs: SortedVec<RubiksCube> = SortedVec::from_array([rc_final.clone()]);
		let mut left_moves: u32 = 0;
		let mut right_moves: u32 = 0;

		let rc_middle = loop {
			println!(
				"left_moves: {left_moves}, right_moves: {right_moves}, left_rcs.len: {}, right_rsc.len: {}",
				left_rcs.items.len(), right_rcs.items.len()
			);

			let mut left_rcs_new: SortedVec<RubiksCube> = SortedVec::new();
			if CORES_N == 1 {
				for rc in left_rcs.items.iter() {
					for rc_new in rc.juxt() {
						left_rcs_new.insert(rc_new);
					}
				}
			}
			else {
				let chunk_size: usize = left_rcs.items.len().div_ceil(CORES_N);
				let left_rcs_new_parts: Vec<SortedVec<RubiksCube>> = left_rcs.items
					.into_par_iter()
					.chunks(chunk_size)
					.map(|rcs| {
						let mut rcs_new: SortedVec<RubiksCube> = SortedVec::new();
						for rc in rcs.iter() {
							for rc_new in rc.juxt() {
								rcs_new.insert(rc_new);
							}
						}
						rcs_new
					})
					.collect();
				left_rcs_new = SortedVec::from_sorted_vecs(left_rcs_new_parts);
			}
			left_moves += 1;
			left_rcs = left_rcs_new;

			println!(
				"left_moves: {left_moves}, right_moves: {right_moves}, left_rcs.len: {}, right_rsc.len: {}",
				left_rcs.items.len(), right_rcs.items.len()
			);

			if let Some(rc_middle) = left_rcs.intersection_with(&right_rcs) {
				break rc_middle
			}

			let mut right_rcs_new: SortedVec<RubiksCube> = SortedVec::new();
			if CORES_N == 1 {
				for rc in right_rcs.items.iter() {
					for rc_new in rc.juxt() {
						right_rcs_new.insert(rc_new);
					}
				}
			}
			else {
				let chunk_size: usize = right_rcs.items.len().div_ceil(CORES_N);
				let right_rcs_new_parts: Vec<SortedVec<RubiksCube>> = right_rcs.items
					.into_par_iter()
					.chunks(chunk_size)
					.map(|rcs| {
						let mut rcs_new: SortedVec<RubiksCube> = SortedVec::new();
						for rc in rcs.iter() {
							for rc_new in rc.juxt() {
								rcs_new.insert(rc_new);
							}
						}
						rcs_new
					})
					.collect();
				right_rcs_new = SortedVec::from_sorted_vecs(right_rcs_new_parts);
			}
			right_moves += 1;
			right_rcs = right_rcs_new;

			if let Some(rc_middle) = left_rcs.intersection_with(&right_rcs) {
				break rc_middle
			}
		};

		// println!("rc_middle:\n{}", rc_middle.to_string1());

		if left_moves + right_moves == 1 {
			assert_eq!(right_moves, 0);
			vec![ALL_MOVES[rc_init.juxt().into_iter().position(|rc| rc == rc_final).unwrap()].clone()]
		}
		else {
			[
				rc_init.solve(&rc_middle),
				rc_middle.solve(&rc_final),
			].concat()
		}
	}

	fn juxt(&self) -> [RubiksCube; 18] {
		[
			{ let mut rc = self.clone(); rc.front(); rc },
			{ let mut rc = self.clone(); rc.front_s(); rc },
			{ let mut rc = self.clone(); rc.back(); rc },
			{ let mut rc = self.clone(); rc.back_s(); rc },
			{ let mut rc = self.clone(); rc.left(); rc },
			{ let mut rc = self.clone(); rc.left_s(); rc },
			{ let mut rc = self.clone(); rc.right(); rc },
			{ let mut rc = self.clone(); rc.right_s(); rc },
			{ let mut rc = self.clone(); rc.top(); rc },
			{ let mut rc = self.clone(); rc.top_s(); rc },
			{ let mut rc = self.clone(); rc.bottom(); rc },
			{ let mut rc = self.clone(); rc.bottom_s(); rc },
			{ let mut rc = self.clone(); rc.x(); rc },
			{ let mut rc = self.clone(); rc.x_s(); rc },
			{ let mut rc = self.clone(); rc.y(); rc },
			{ let mut rc = self.clone(); rc.y_s(); rc },
			{ let mut rc = self.clone(); rc.z(); rc },
			{ let mut rc = self.clone(); rc.z_s(); rc },
		]
	}

	fn front(&mut self) {
		self.pieces.rotate4(12, 14, 38, 36);
		self.pieces.rotate4(13, 26, 37, 24);
		self.pieces.rotate4(6, 15, 47, 35);
		self.pieces.rotate4(7, 27, 46, 23);
		self.pieces.rotate4(8, 39, 45, 11);
	}
	fn front_s(&mut self) {
		self.pieces.rotate4(12, 36, 38, 14);
		self.pieces.rotate4(13, 24, 37, 26);
		self.pieces.rotate4(6, 35, 47, 15);
		self.pieces.rotate4(7, 23, 46, 27);
		self.pieces.rotate4(8, 11, 45, 39);
	}
	fn back(&mut self) {
		self.pieces.rotate4(18, 20, 44, 42);
		self.pieces.rotate4(19, 32, 43, 30);
		self.pieces.rotate4(0, 33, 53, 17);
		self.pieces.rotate4(1, 21, 52, 29);
		self.pieces.rotate4(2, 9, 51, 41);
	}
	fn back_s(&mut self) {
		self.pieces.rotate4(18, 42, 44, 20);
		self.pieces.rotate4(19, 30, 43, 32);
		self.pieces.rotate4(0, 17, 53, 33);
		self.pieces.rotate4(1, 29, 52, 21);
		self.pieces.rotate4(2, 41, 51, 9);
	}
	fn left(&mut self) {
		self.pieces.rotate4(9, 11, 35, 33);
		self.pieces.rotate4(10, 23, 34, 21);
		self.pieces.rotate4(0, 12, 45, 44);
		self.pieces.rotate4(3, 24, 48, 32);
		self.pieces.rotate4(6, 36, 51, 20);
	}
	fn left_s(&mut self) {
		self.pieces.rotate4(9, 33, 35, 11);
		self.pieces.rotate4(10, 21, 34, 23);
		self.pieces.rotate4(0, 44, 45, 12);
		self.pieces.rotate4(3, 32, 48, 24);
		self.pieces.rotate4(6, 20, 51, 36);
	}
	fn right(&mut self) {
		self.pieces.rotate4(15, 17, 41, 39);
		self.pieces.rotate4(16, 29, 40, 27);
		self.pieces.rotate4(8, 18, 53, 38);
		self.pieces.rotate4(5, 30, 50, 26);
		self.pieces.rotate4(2, 42, 47, 14);
	}
	fn right_s(&mut self) {
		self.pieces.rotate4(15, 39, 41, 17);
		self.pieces.rotate4(16, 27, 40, 29);
		self.pieces.rotate4(8, 38, 53, 18);
		self.pieces.rotate4(5, 26, 50, 30);
		self.pieces.rotate4(2, 14, 47, 42);
	}
	fn top(&mut self) {
		self.pieces.rotate4(0, 2, 8, 6);
		self.pieces.rotate4(1, 5, 7, 3);
		self.pieces.rotate4(12, 9, 18, 15);
		self.pieces.rotate4(13, 10, 19, 16);
		self.pieces.rotate4(14, 11, 20, 17);
	}
	fn top_s(&mut self) {
		self.pieces.rotate4(0, 6, 8, 2);
		self.pieces.rotate4(1, 3, 7, 5);
		self.pieces.rotate4(12, 15, 18, 9);
		self.pieces.rotate4(13, 16, 19, 10);
		self.pieces.rotate4(14, 17, 20, 11);
	}
	fn bottom(&mut self) {
		self.pieces.rotate4(45, 47, 53, 51);
		self.pieces.rotate4(46, 50, 52, 48);
		self.pieces.rotate4(36, 39, 42, 33);
		self.pieces.rotate4(37, 40, 43, 34);
		self.pieces.rotate4(38, 41, 44, 35);
	}
	fn bottom_s(&mut self) {
		self.pieces.rotate4(45, 51, 53, 47);
		self.pieces.rotate4(46, 48, 52, 50);
		self.pieces.rotate4(36, 33, 42, 39);
		self.pieces.rotate4(37, 34, 43, 40);
		self.pieces.rotate4(38, 35, 44, 41);
	}
	fn x(&mut self) {
		self.left_s();
		self.right();
	}
	fn x_s(&mut self) {
		self.left();
		self.right_s();
	}
	fn y(&mut self) {
		self.top_s();
		self.bottom();
	}
	fn y_s(&mut self) {
		self.top();
		self.bottom_s();
	}
	fn z(&mut self) {
		self.front_s();
		self.back();
	}
	fn z_s(&mut self) {
		self.front();
		self.back_s();
	}

	fn to_string1(&self) -> String {
		let [_00, _01, _02, _03, _04, _05, _06, _07, _08, _09, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19, _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38, _39, _40, _41, _42, _43, _44, _45, _46, _47, _48, _49, _50, _51, _52, _53] = self.pieces;
		[
			format!("      {_00:?} {_01:?} {_02:?}\n"),
			format!("      {_03:?} {_04:?} {_05:?}\n"),
			format!("      {_06:?} {_07:?} {_08:?}\n"),
			format!("{_09:?} {_10:?} {_11:?} {_12:?} {_13:?} {_14:?} {_15:?} {_16:?} {_17:?} {_18:?} {_19:?} {_20:?}\n"),
			format!("{_21:?} {_22:?} {_23:?} {_24:?} {_25:?} {_26:?} {_27:?} {_28:?} {_29:?} {_30:?} {_31:?} {_32:?}\n"),
			format!("{_33:?} {_34:?} {_35:?} {_36:?} {_37:?} {_38:?} {_39:?} {_40:?} {_41:?} {_42:?} {_43:?} {_44:?}\n"),
			format!("      {_45:?} {_46:?} {_47:?}\n"),
			format!("      {_48:?} {_49:?} {_50:?}\n"),
			format!("      {_51:?} {_52:?} {_53:?}\n"),
		].concat()
	}
}



// trait ExtArrayRotate {
// 	fn rotate<const N: usize>(&mut self, indices: [u8; N]);
// }
// impl<T: Copy, const L: usize> ExtArrayRotate for [T; L] {
// 	// #[inline] // TODO: test
// 	fn rotate<const N: usize>(&mut self, indices: [u8; N]) {
// 		let tmp = self[indices[indices.len()-1] as usize];
// 		// dbg!(tmp);
// 		for i in (0..indices.len()-1).rev() {
// 			let index_l = indices[i+1] as usize;
// 			let index_r = indices[i] as usize;
// 			// dbg!(index_l, index_r);
// 			self[index_l] = self[index_r];
// 			// dbg!(&self);
// 		}
// 		self[indices[0] as usize] = tmp;
// 	}
// }

trait ExtArrayRotate4 {
	fn rotate4(&mut self, i1: usize, i2: usize, i3: usize, i4: usize);
}
impl<T: Copy, const L: usize> ExtArrayRotate4 for [T; L] {
	// #[inline] // TODO: test
	fn rotate4(&mut self, i1: usize, i2: usize, i3: usize, i4: usize) {
		let temp = self[i4];
		self[i4] = self[i3];
		self[i3] = self[i2];
		self[i2] = self[i1];
		self[i1] = temp;
	}
}



#[derive(Debug, Clone)]
struct SortedVec<T: Clone + PartialOrd + Ord> {
	items: Vec<T>,
}
impl<T: Clone + PartialOrd + Ord> SortedVec<T> {
	fn new() -> Self {
		Self { items: vec![] }
	}

	fn from_vec(mut items: Vec<T>) -> Self {
		items.sort();
		Self { items }
	}

	fn from_array<const N: usize>(items: [T; N]) -> Self {
		let mut items = items.to_vec();
		items.sort();
		Self { items }
	}

	fn from_sorted_vecs(sorted_vecs: Vec<SortedVec<T>>) -> Self {
		let mut heap = BinaryHeap::new();

		// Keep track of iterators for each vec
		let mut iters: Vec<_> = sorted_vecs.into_iter()
			.map(|sv| sv.items.into_iter())
			.collect();

		// Initialize the heap with the first element from each iterator
		for (i, iter) in iters.iter_mut().enumerate() {
			if let Some(value) = iter.next() {
				heap.push(Reverse((value, i)));
			}
		}

		let mut merged = Vec::new();

		while let Some(Reverse((value, i))) = heap.pop() {
			merged.push(value);
			if let Some(next) = iters[i].next() {
				heap.push(Reverse((next, i)));
			}
		}

		SortedVec { items: merged }
	}

	fn insert(&mut self, item: T) {
		// dbg!(self.index_of(&item));
		if let Err(index) = self.index_of(&item) {
			self.items.insert(index, item);
		}
	}

	/// returns `Ok(index where it is)` or `Err(index before which it should be)`.
	fn index_of(&self, target: &T) -> Result<usize, usize> {
		let mut l = 0;
		let mut r = self.items.len();
		while l < r {
			let m = l + (r - l) / 2;
			match self.items[m].cmp(&target) {
				Ordering::Equal   => return Ok(m),
				Ordering::Less    => { l = m + 1 }
				Ordering::Greater => { r = m }
			}
		}
		debug_assert_eq!(l, r);
		Err(l)
	}

	fn intersection_with(&self, other: &Self) -> Option<T> {
		let mut index_l = 0;
		let mut index_r = 0;
		while index_l < self.items.len() && index_r < other.items.len() {
			match self.items[index_l].cmp(&other.items[index_r]) {
				Ordering::Equal => return Some(self.items[index_l].clone()),
				Ordering::Less    => { index_l += 1; }
				Ordering::Greater => { index_r += 1; }
			}
		}
		None
	}
}



trait ExtResultCollapse<T> {
	fn collapse(self) -> T;
}
impl<T> ExtResultCollapse<T> for Result<T, T> {
	fn collapse(self) -> T {
		match self {
			Ok(v) => v,
			Err(e) => e,
		}
	}
}



#[cfg(test)]
mod rubiks_cube {
	use super::*;
	mod solve {
		use super::*;
		mod moves {
			use super::*;
			mod _1 {
				use super::*;
				#[test]
				fn front() {
					let mut rc = RubiksCube::new();
					rc.front_s();
					assert_eq!(
						vec![Move::Front],
						rc.solve(&RubiksCube::new())
					)
				}

			}
		}
	}
	mod moves {
		use super::*;
		use Color::*;
		#[test]
		fn front_solved() {
			let mut rc = RubiksCube::new();
			rc.front();
			assert_eq!(
				RubiksCube::from([
						  Y,Y,Y,
						  Y,Y,Y,
						  B,B,B,
					B,B,W,R,R,R,Y,G,G,O,O,O,
					B,B,W,R,R,R,Y,G,G,O,O,O,
					B,B,W,R,R,R,Y,G,G,O,O,O,
						  G,G,G,
						  W,W,W,
						  W,W,W,
				]),
				rc
			)
		}
	}
	mod to_string1 {
		use super::*;
		#[test]
		fn new() {
			let expected = [
				"      Y Y Y\n",
				"      Y Y Y\n",
				"      Y Y Y\n",
				"B B B R R R G G G O O O\n",
				"B B B R R R G G G O O O\n",
				"B B B R R R G G G O O O\n",
				"      W W W\n",
				"      W W W\n",
				"      W W W\n",
			].concat();
			let actual = RubiksCube::new().to_string1();
			println!("expected:\n{expected}");
			println!("actual:\n{actual}");
			assert_eq!(expected, actual)
		}
	}
}

#[test]
fn rotate4() {
	//              0    1    2    3    4    5    6    7
	let mut arr = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
	arr.rotate4(1, 4, 5, 6);
	assert_eq!(
		['a', 'g', 'c', 'd', 'b', 'e', 'f', 'h'],
		arr
	);
}
