mod actor;
mod licences;

use std::collections::HashSet;
use std::io::Write;

use self::actor::Actor;
use self::licences::{Licences, SkillType, Weapon};

const TURN: usize = 5;
type Pattern = [u8; TURN];

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let dict = Licences::load("data/licences.json")?;
	println!("loaded {} weapons", dict.len());

	print!("select your weapon: ");
	std::io::stdout().flush()?;

	let mut input = String::new();
	std::io::stdin().read_line(&mut input)?;
	let weapon = dict.get(input.trim()).ok_or("invalid weapon")?;

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
			print!("{}, ", weapon.skill(idx as usize).name);
		}
		println!();
	}

	Ok(())
}

fn simulate_duel(p1_weapon: &Weapon, p1_pattern: &Pattern, p2_weapon: &Weapon, p2_pattern: &Pattern) -> (bool, i32, i32) {
	const VOLTAGE: [f32; 5] = [1.0, 1.1, 1.25, 1.5, 2.0];
	let (mut p1_voltage, mut p2_voltage) = (HashSet::new(), HashSet::new());
	let (mut p1_score, mut p2_score) = (0, 0);
	for i in 0..TURN {
		let p1 = p1_weapon.skill(p1_pattern[i] as usize);
		let p2 = p2_weapon.skill(p2_pattern[i] as usize);
		let p1_damage = match p1.ty {
			SkillType::Sp | SkillType::Ex => (p1.atk as f32 * VOLTAGE[p1_voltage.len()]) as i32,
			_ => p1.atk,
		};
		let p2_damage = match p2.ty {
			SkillType::Sp | SkillType::Ex => (p2.atk as f32 * VOLTAGE[p2_voltage.len()]) as i32,
			_ => p2.atk,
		};
		match p1.ty.cmp(&p2.ty) {
			std::cmp::Ordering::Greater => {
				p1_score += p1_damage - p2.def;
			}
			std::cmp::Ordering::Less => {
				p2_score += p2_damage - p1.def;
			}
			std::cmp::Ordering::Equal => {
				p1_score += p1_damage / 2 - p2.def;
				p2_score += p2_damage / 2 - p1.def;
			}
		}
		p1_voltage.insert(p1.ty);
		p2_voltage.insert(p2.ty);
	}
	(p1_score > p2_score, p1_score, p2_score)
}
