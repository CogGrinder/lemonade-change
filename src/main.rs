// use std::any::Any;
// use itertools::Itertools;
use diam::prelude::*;
use rayon::join;
// use diam::join;
use rayon::prelude::*;

const N: usize = 10_000_000;
const N_BAD: usize = 10;


#[derive(Copy, Clone)]
// for explicit copying to prevent moving instead of copying
struct Change {
	// required_fives: u32,
	// required_tens: u32,

	out_fives: u32,
	out_tens: u32,

	// index ?
}

impl Change {
	fn new() -> Self {
		Change {
			// required_fives: 0,
			// required_tens: 0,

			out_fives: 0,
			out_tens: 0,


		}
	}
}


fn fusion(c1:Option<Change>, c2:Option<Change>) -> Option<Change> {
	if c1.is_none() || c2.is_none() {
		// this case is not evaluated here
		None
	} else {
		Some(Change{
					out_fives: c1.unwrap().out_fives +  c2.unwrap().out_fives,
					out_tens: c1.unwrap().out_tens +  c2.unwrap().out_tens,
				})
	}
}

/*
 * Idea: inner_par_can_provide_change handles folding
 * recursive calls of inner_can_provide_change make recursive folds
 */
fn seq_can_provide_change(s: &[u32],verbose:Option<bool>) -> bool {
	inner_seq_using_join_can_provide_change(s,s.len(),Some(Change::new()),Some(verbose.unwrap_or(false))).is_some()
}



fn par_can_provide_change(s: &[u32],verbose:Option<bool>) -> bool {
	let levels = 3;

	// parallel splitting of s and fusion in inner function
	// no "or" fusion here, it is in inner function
	inner_level_abstraction_par_can_provide_change(s,s.len(), Some(Change::new()),levels,Some(verbose.unwrap_or(false))).is_some()
}

fn inner_level_abstraction_par_can_provide_change(s: &[u32], length: usize, change: Option<Change>, levels: usize, verbose:Option<bool>) -> Option<Change> {
	if levels <= 0 || length<=2 {
		inner_seq_using_join_can_provide_change(s,length,change.clone(),Some(verbose.unwrap_or(false)))
    } else {

		let mid = (length+1) / 2; // TODO: try 2/3 instead of 1/2
		if verbose.unwrap_or(false) {println!("LEVEL {levels}: {mid} {:?} {:?}",&s[..mid],&s[mid..length])};
		let (c1, c2) = join(
			|| inner_level_abstraction_par_can_provide_change(&s[..],mid,
				change,levels-1,
				Some(verbose.unwrap_or(false))),
			|| inner_level_abstraction_par_can_provide_change(&s[mid..],length-mid,
				Some(Change::new()),levels-1,
				Some(verbose.unwrap_or(false))),
		);

		if verbose.unwrap_or(false) {
			println!("c1:{} 5s: {}, 10s: {}",
			c1.is_some(), if c1.is_some() {c1.unwrap().out_fives} else {0}, if c1.is_some() {c1.unwrap().out_tens} else {0});
			println!("c2:{} 5s: {}, 10s: {}",
			c2.is_some(), if c2.is_some() {c2.unwrap().out_fives} else {0}, if c2.is_some() {c2.unwrap().out_tens} else {0});
			println!();
		}
		

		if c1.is_none() {
			// customers that led to c1 are before c2 so c1 must be valid for fusion
			None
		} else if c2.is_none() {
			// continue the execution of c1 if there is leftover change
			
			// TODO: replace if with conditional for loop
			if c1.unwrap().out_fives>0 || c1.unwrap().out_tens>0 {
				// TODO: for better accuracy, might need to use required_fives and required_tens
				// with corresponding out_fives and out_tens (multiple branches)
				if verbose.unwrap_or(false) {println!(" c1 rewriting c2... 5s: {}, 10s: {}",c1.unwrap().out_fives, c1.unwrap().out_tens)};
				inner_level_abstraction_par_can_provide_change(&s[mid..],length-mid,
					c1.clone(),levels-1,
					Some(verbose.unwrap_or(false)))

			} else {
				None
			}
		} else {
			if verbose.unwrap_or(false) {
				println!("fusioning... c1 5s {}, 10s {} ; c2 5s {}, 10s {}",
					c1.unwrap().out_fives, c1.unwrap().out_tens, c2.unwrap().out_fives, c2.unwrap().out_tens)
				};
			fusion(c1, c2)
		}
		//.fold(Some(true), |is_change: Option<bool>, e: &u32| { etc
		// make an or here? no it is in inner function
	}
}


/*
 * Idea: inner_can_provide_change branches to a new thread every time it needs to give change for a 20
 */
fn inner_seq_using_join_can_provide_change(s: &[u32], length: usize, change: Option<Change>,verbose:Option<bool>) -> Option<Change> {
	// if verbose.unwrap_or(false) {println!("  {:?}/{:?}",&s[0..length],&s[0..]);}
	if verbose.unwrap_or(false) {println!("  {:?}/{:?} {:?}",length,s.len(),&s[0..]);}
	if change.is_none() {
		// note: this would occur after making a None change
		// therefore may be useless after try_fold is implemented
		None
	} else {
		s[0..length].into_iter().enumerate()
		.try_fold(change.unwrap().clone(), |mut incoming_change, (i,e)| {
			if verbose.unwrap_or(false) {println!("  i and el: {i}, {e} ; 5s: {}, 10s: {}",incoming_change.out_fives, incoming_change.out_tens)};
			match *e {
				5 => {
					incoming_change.out_fives+=1;
					Some(incoming_change)
				},
				10 => {
					if incoming_change.out_fives == 0 {
						None
					} else {
						incoming_change.out_fives-=1;
						incoming_change.out_tens+=1;
						Some(incoming_change)
					}
				},
				20 => {
					let can_do_3_fives = incoming_change.out_fives >= 3;
					let can_do_1_five_1_ten = incoming_change.out_fives >= 1 && incoming_change.out_tens >= 1;
					
					// thank you to https://users.rust-lang.org/t/match-vectors-by-content/102603/4
					let cases_vector = [can_do_3_fives, can_do_1_five_1_ten];

					// 3 cases : no way through, 2 ways through, 1 way through
					match cases_vector {
						// both branches are valid
						[true,true] => {
							// branching occurs
							
							// create 2 "change" variants
							let mut new_change_3_fives = incoming_change.clone();
							new_change_3_fives.out_fives-=3;
		
							let mut new_change_1_five_1_ten = incoming_change.clone();
							new_change_1_five_1_ten.out_fives-=1;
							new_change_1_five_1_ten.out_tens-=1;
		
							let (lh,rh) = join(
								// branch 3 fives
								// TODO: shorten s
								|| {
									if verbose.unwrap_or(false) {println!("20-branch - new_change_3_fives, {} {}",length-i-1,s.len()-i-1)};
									inner_seq_using_join_can_provide_change(&s[i+1..],std::cmp::min(length-i-1,s.len()-i-1),Some(new_change_3_fives),Some(verbose.unwrap_or(false)))
								},
								// branch 2 fives and a ten
								|| {
									if verbose.unwrap_or(false) {println!("20-branch - new_change_1_five_1_ten, {} {}",length-i-1,s.len()-i-1)};
									inner_seq_using_join_can_provide_change(&s[i+1..],std::cmp::min(length-i-1,s.len()-i-1),Some(new_change_1_five_1_ten),Some(verbose.unwrap_or(false)))
								},
							);
		
							//or is here:
							if lh.is_some() {
								// default change is 3 fives TODO: debug
								Some(new_change_3_fives)
							} else if rh.is_some() {
								// if lh doesnt work fall back to rh
								Some(new_change_1_five_1_ten)
							} else {
								None
							}
						},

						// one branch valid:
						// -> only 3 fives branch is valid
						[true,false] => {
							incoming_change.out_fives-=3;
							Some(incoming_change)
						},
						// -> only 1 five and 1 ten branch is valid
						[false,true] => {
							incoming_change.out_fives-=1;
							incoming_change.out_tens-=1;
							Some(incoming_change)
						},

						// no branches are valid
						[false,false] => {
							None
						},
					}
				}
				_ => {
					None
				},
			}
		})
	}

}

/*
 * Provides a generic testing formula
 * Thank you to https://stackoverflow.com/questions/36390665/how-do-you-pass-a-rust-function-as-a-parameter
 */
fn testing(function_name:&str, can_provide_change_variant: &dyn Fn(&[u32],Option<bool>) -> bool ) {
	println!("Simple tests...");
	let debug = Some(false);
	
	let s:[u32; 1] = [5];
	let result_seq = can_provide_change_variant(&s,debug);
	assert_eq!(result_seq,true);
	
	let s:[u32; 1] = [10];
	let result_seq = can_provide_change_variant(&s,debug);
	assert_eq!(result_seq,false);
	
	let s:[u32; 1] = [20];
	let result_seq = can_provide_change_variant(&s,debug);
	assert_eq!(result_seq,false);
	
	let s:[u32; 2] = [5,10];
	let result_seq = can_provide_change_variant(&s,debug);
	assert_eq!(result_seq,true);
	
	let s:[u32; 3] = [5,10,10];
	let result_seq = can_provide_change_variant(&s,debug);
	assert_eq!(result_seq,false);
	
	println!("Major tests:");
	let debug = Some(false);
	
	let s:[u32; 4] = [5,5,10,20];
	let start = std::time::Instant::now();
	let result_seq = can_provide_change_variant(&s,debug);
	println!("{function_name} took {:?} on {:?}", start.elapsed(),s);
	// solution : give 1 five and 1 ten for 20
	assert_eq!(result_seq,true);
	
	let s:[u32; 4] = [5,5,5,20];
	let start = std::time::Instant::now();
	let result_seq = can_provide_change_variant(&s,debug);
	println!("{function_name} took {:?} on {:?}", start.elapsed(),s);
	// solution : give 3 fives for 20
	assert_eq!(result_seq,true);
	
	let s:[u32; 3] = [5,5,20];
	let start = std::time::Instant::now();
	let result_seq = can_provide_change_variant(&s,debug);
	println!("{function_name} took {:?} on {:?}", start.elapsed(),s);
	// no possible solutions for 20
	assert_eq!(result_seq,false);
	
	let s:[u32; 7] = [5,5,5,5,10,20,5];
	let start = std::time::Instant::now();
	let result_seq = can_provide_change_variant(&s,debug);
	println!("{function_name} took {:?} on {:?}", start.elapsed(),s);
	// two possible solutions for 20, followed by simple 5
	assert_eq!(result_seq,true);
	
	let s:[u32; 14] = [5,5,5,5,10,20,5,5,5,5,5,10,20,5];
	let start = std::time::Instant::now();
	let result_seq = can_provide_change_variant(&s,debug);
	println!("{function_name} took {:?} on {:?}", start.elapsed(),s);
	// two possible solutions for 20, followed by simple 5
	assert_eq!(result_seq,true);
	
	let debug = Some(true);
	let s:[u32; 7] = [5,5,5,5,10,20,10];
	let start = std::time::Instant::now();
	let result_seq = can_provide_change_variant(&s,debug);
	println!("{function_name} took {:?} on {:?}", start.elapsed(),s);
	// two possible solutions for 20, followed by less simple 10
	assert_eq!(result_seq,true);
	
	let s:[u32; 12] = [5,5,5,5,10,20,5,5,5,5,10,20];
	let start = std::time::Instant::now();
	let result_seq = can_provide_change_variant(&s,debug);
	println!("{function_name} took {:?} on {:?}", start.elapsed(),s);
	// two possible solutions for 20, twice
	assert_eq!(result_seq,true);
	
	let debug = Some(true);
	let s:[u32; 10] = [5, 5, 5, 10, 5, 20, 5, 10, 20, 10];
	let start = std::time::Instant::now();
	let result_seq = can_provide_change_variant(&s,debug);
	println!("{function_name} took {:?} on {:?}", start.elapsed(),s);
	// two possible solutions for 20, twice
	assert_eq!(result_seq,true);
}

fn main() {

	testing("seq_can_provide_change",&seq_can_provide_change);
	// testing("par_can_provide_change",&par_can_provide_change);
	
	println!("\nTesting randomly generated yes- and no-instances of size {N}...");
	// randomly generated lemonade queue
	// map gives a distribution for likely yes and no instances
	let even_map = [5,10,20];
	let uneven_map = [5,5,5,5,10,10,20];

	use rand::Rng;
	let mut rng = rand::thread_rng();
	
	/*
	* no-instance
	*/
	let start = std::time::Instant::now();
    let s: Vec<u32> = std::iter::repeat_with(|| even_map[rng.gen_range(0..even_map.len())]).take(N).collect();
    println!("\n -> No-instance generation of size {N} took {:?}", start.elapsed());

	let debug = Some(false);

	let start = std::time::Instant::now();
	let result_seq = seq_can_provide_change(&s,debug);
	println!("seq_can_provide_change took {:?} on randomly generated lemonade queue", start.elapsed());
	// two possible solutions for 20, twice
	println!(" Output: {result_seq}");

	let start = std::time::Instant::now();
	let result = par_can_provide_change(&s,debug);
	println!("par_can_provide_change took {:?} on randomly generated lemonade queue", start.elapsed());
	// two possible solutions for 20, twice
	println!(" Output: {result}");

	/*
	 * likely-yes-instance
	 * TODO: note the extreme variance due to 20 bills making branchings that are re-computed
	 * Likely solution: add better branching
	 */
	let start = std::time::Instant::now();
	
    let mut s: Vec<u32> = std::iter::repeat_with(|| uneven_map[rng.gen_range(0..uneven_map.len())]).take(N_BAD).collect();
	for i in 0..(N_BAD>>3)  {
		s[i]=5;
	}
	println!("\n -> Likely-yes-instance generation of size {N_BAD} took {:?}", start.elapsed());
	let debug = Some(true);
	if debug.unwrap_or(false) {println!("{:?}",s)};
	let debug = Some(false);

	let start = std::time::Instant::now();
	let result_seq = seq_can_provide_change(&s,debug);
	// let mut result_seq = true;
	// diam::svg("seq_can_provide_change.svg", || {
    //     result_seq = seq_can_provide_change(&s,debug);
    // })
    // .expect("failed saving svg file");
	
	println!("seq_can_provide_change took {:?} on randomly generated lemonade queue", start.elapsed());
	// two possible solutions for 20, twice
	println!(" Output: {result_seq}");

	let start = std::time::Instant::now();
	let result = par_can_provide_change(&s,debug);
	// let mut result = true;
	// diam::svg("par_can_provide_change.svg", || {
    //     result = par_can_provide_change(&s,debug);
    // })
    // .expect("failed saving svg file");

	println!("par_can_provide_change took {:?} on randomly generated lemonade queue", start.elapsed());
	// two possible solutions for 20, twice
	println!(" Output: {result}");

}
