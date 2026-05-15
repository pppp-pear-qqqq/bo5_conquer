mod actor;
mod error;
mod licences;

use std::collections::HashSet;
use std::io::{self, Write};

use clap::{Parser, Subcommand};

use self::actor::Actor;
use self::error::Error;
use self::licences::{Licences, SkillType, Weapon};

const TURN: usize = 5;
type Pattern = [u8; TURN];

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
	#[command(subcommand)]
	mode: Option<Mode>,
	#[arg(short, long)]
	weapon: Option<String>,
}
#[derive(Subcommand)]
enum Mode {
	Find {
		#[arg(short, long)]
		opponent: Option<String>,
		#[arg(short, long)]
		min_rate: Option<f32>,
	},
	Simulate {
		#[arg(short, long)]
		op_weapon: Option<String>,
		al_pattern: Option<String>,
		op_pattern: Option<String>,
	},
	Registration,
}

fn main() -> Result<(), self::error::Error> {
	let prompt_input = |msg: &str| -> Result<String, io::Error> {
		print!("{}", msg);
		std::io::stdout().flush()?;
		let mut input = String::new();
		std::io::stdin().read_line(&mut input)?;
		Ok(input.trim().to_string())
	};

	let args = Args::parse();

	let dict = Licences::load("data/licences.json")?;
	println!("loaded {} weapons", dict.len());

	let mode = match args.mode {
		Some(mode) => mode,
		None => {
			let input = prompt_input("select mode([f]ind/[s]imulate): ")?;
			match input.as_str() {
				"f" | "find" => Mode::Find { opponent: None, min_rate: None },
				"s" | "simulate" => Mode::Simulate { op_weapon: None, al_pattern: None, op_pattern: None },
				_ => return Err(Error::InvalidInput(input)),
			}
		}
	};

	let weapon = dict.get(&args.weapon.unwrap_or(prompt_input("select your weapon: ")?))?;

	match mode {
		Mode::Find { opponent, min_rate } => {
			let opponent = Actor::load(format!("data/patterns/{}.json", opponent.unwrap_or(prompt_input("select your opponent: ")?)), &dict)?;
			let min_rate = min_rate.unwrap_or(prompt_input("min rate: ")?.parse().map_err(|_| Error::InvalidInput("{rate}".into()))?);

			let results = opponent.find_pattern(weapon, min_rate);
			for (score, pattern) in results {
				print!("{:.2}\t", score);
				for i in 0..pattern.len() {
					print!("{}", weapon.skill(pattern[i] as usize).name);
					if i < pattern.len() - 1 {
						print!(", ");
					}
				}
				println!();
			}

			Ok(())
		}
		Mode::Simulate { op_weapon, al_pattern, op_pattern } => {
			let op_weapon = dict.get(&op_weapon.unwrap_or(prompt_input("select opponent weapon: ")?))?;
			let al_pattern = parse_pattern(&al_pattern.unwrap_or(prompt_input("your pattern: ")?))?;
			let op_pattern = parse_pattern(&op_pattern.unwrap_or(prompt_input("opponent pattern: ")?))?;

			let (result, p1_score, p2_score) = simulate_duel(&weapon, &al_pattern, op_weapon, &op_pattern);
			println!("win: {}\n({} : {})", if result { "left" } else { "right" }, p1_score, p2_score);
			Ok(())
		}
		Mode::Registration => todo!(),
	}
}

fn parse_pattern(input: &str) -> Result<Pattern, Error> {
	let parts: Vec<&str> = input.trim().split_whitespace().collect();
	if parts.len() != TURN {
		return Err(Error::InvalidInput(format!("require {} numbers", TURN)));
	}
	let mut pattern = [0; TURN];
	for i in 0..TURN {
		pattern[i] = parts[i].parse::<u8>().map_err(|e| Error::InvalidInput(e.to_string()))?;
	}
	Ok(pattern)
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
