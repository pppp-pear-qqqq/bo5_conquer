mod licences;

use std::io::Write;

use self::licences::Licences;

const TURN: usize = 5;
type Pattern = [u8; TURN];

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let dict = Licences::load("data/licences.json")?;
	println!("loaded {} weapons", dict.len());

	print!("select your weapon: ");
	std::io::stdout().flush()?;

	let mut input = String::new();
	std::io::stdin().read_line(&mut input)?;
	let weapon = dict.get(input.trim());

	if let Some(weapon) = weapon {
		print!("select your opponent: ");
		std::io::stdout().flush()?;
		let mut input = String::new();
		std::io::stdin().read_line(&mut input)?;
		let opponent = Actor::load(format!("data/patterns/{}.json", input.trim()), &dict)?;

		print!("min rate: ");
		std::io::stdout().flush()?;
		let mut input = String::new();
		std::io::stdin().read_line(&mut input)?;
		let min_win_rate = input.trim().parse::<f32>()?;

		let results = opponent.find_pattern(weapon, min_win_rate);
		for (score, pattern) in results {
			print!("{:.2}\t", score);
			for idx in pattern {
				print!("{}, ", weapon.skill_list[idx as usize].name);
			}
			println!();
		}
	}

	Ok(())
}
