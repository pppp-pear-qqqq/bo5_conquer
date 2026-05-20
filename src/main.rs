mod actor;
mod command;
mod error;
mod licences;

use std::io::{self, Write};
use std::str::FromStr;

use clap::{Parser, Subcommand};

use crate::error::Error;

const ROUND: usize = 5;
type Pattern = [u8; ROUND];

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
	#[command(subcommand)]
	mode: Option<Mode>,
	#[arg(short, long, global = true)]
	out_dir: Option<String>,
	#[arg(long, global = true, default_value = "false")]
	overwrite: bool,
}
#[derive(Subcommand)]
enum Mode {
	// 1件探索
	Find {
		#[arg(short, long)]
		weapon: Option<String>,
		#[arg(short, long)]
		opponent: Option<String>,
		#[arg(short, long)]
		min_score: Option<f32>,
		#[arg(short, long, default_value = "true")]
		allow_draw: bool,
	},
	// 全相手をまとめて
	FindAll {
		#[arg(short, long)]
		weapon: Option<String>,
		#[arg(short, long, default_value = "data/patterns")]
		input_dir: String,
		#[arg(short, long, default_value = "1.0")]
		min_score: f32,
		#[arg(short, long, default_value = "false")]
		allow_draw: bool,
	},
	// 一貫するパターン探索
	Consistents {
		#[arg(short, long)]
		weapon: Option<String>,
		#[arg(short, long)]
		input_dir: Option<String>,
		#[arg(short, long, default_value = "false")]
		recursive: bool,
	},
	// シミュレーション
	Simulate {
		#[arg(short, long)]
		al_weapon: Option<String>,
		#[arg(short, long)]
		op_weapon: Option<String>,
	},
}

trait OptionUnwrap {
	type Output;
	fn unwrap_or_input(self, prompt: &str) -> Result<Self::Output, io::Error>;
}
impl<T: FromStr> OptionUnwrap for Option<T> {
	type Output = T;
	fn unwrap_or_input(self, prompt: &str) -> Result<T, io::Error> {
		match self {
			Some(value) => Ok(value),
			None => {
				print!("{}", prompt);
				io::stdout().flush()?;
				let mut input = String::new();
				io::stdin().read_line(&mut input)?;
				Ok(input.trim().parse::<T>().map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, input))?)
			}
		}
	}
}
impl FromStr for Mode {
	type Err = io::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"f" | "find" => Self::Find {
				weapon: None,
				opponent: None,
				min_score: None,
				allow_draw: true,
			},
			"a" | "find-all" => Self::FindAll {
				weapon: None,
				input_dir: "data/patterns".to_string(),
				min_score: 1.0,
				allow_draw: false,
			},
			"c" | "consistents" => Self::Consistents {
				weapon: None,
				input_dir: None,
				recursive: false,
			},
			"s" | "simulate" => Self::Simulate { al_weapon: None, op_weapon: None },
			_ => return Err(io::Error::new(io::ErrorKind::InvalidInput, s)),
		})
	}
}

enum Output {
	File { dir: String, overwrite: bool },
	Stdout,
}
impl Output {
	fn make_dir(&self) -> Result<(), io::Error> {
		match self {
			Self::File { dir, .. } => std::fs::create_dir_all(dir),
			Self::Stdout => Ok(()),
		}
	}
	fn gen_write(&self, file_name: &str) -> Result<Box<dyn io::Write>, io::Error> {
		match self {
			Self::File { dir, overwrite } => {
				let full_path = format!("{dir}/{file_name}");
				if *overwrite || !std::fs::exists(&full_path)? {
					Ok(Box::new(std::fs::File::create(&full_path)?))
				} else {
					Err(io::Error::new(io::ErrorKind::AlreadyExists, full_path))
				}
			}
			Self::Stdout => Ok(Box::new(io::stdout())),
		}
	}
}

impl Mode {
	fn input(self) -> Result<Self, io::Error> {
		match self {
			Self::Find {
				weapon,
				opponent,
				min_score,
				allow_draw,
			} => Ok(Self::Find {
				weapon: Some(weapon.unwrap_or_input("select your weapon: ")?),
				opponent: Some(opponent.unwrap_or_input("select opponent: ")?),
				min_score: Some(min_score.unwrap_or_input("min rate: ")?),
				allow_draw,
			}),
			Self::FindAll {
				weapon,
				input_dir,
				min_score,
				allow_draw,
			} => Ok(Self::FindAll {
				weapon: Some(weapon.unwrap_or_input("select your weapon: ")?),
				input_dir,
				min_score,
				allow_draw,
			}),
			Self::Consistents { weapon, input_dir, recursive } => Ok(Self::Consistents {
				weapon: Some(weapon.unwrap_or_input("input your weapon id: ")?),
				input_dir: Some(input_dir.unwrap_or_input("patterns dir: ")?),
				recursive,
			}),
			Self::Simulate { al_weapon, op_weapon } => Ok(Self::Simulate {
				al_weapon: Some(al_weapon.unwrap_or_input("select your weapon: ")?),
				op_weapon: Some(op_weapon.unwrap_or_input("select opponent's weapon: ")?),
			}),
		}
	}
}

fn main() -> Result<(), Error> {
	let args = Args::parse();

	// ライセンス読み込み
	let dict = licences::Licences::load("data/licences.json")?;
	println!("loaded {} weapons", dict.len());

	// 出力設定
	let out = args.out_dir.map(|dir| Output::File { dir, overwrite: args.overwrite }).unwrap_or(Output::Stdout);

	// モード分岐
	let mode = args.mode.unwrap_or_input("select mode([f]ind/[a]ll-find/[c]onsistents/[s]imulate): ")?.input()?;
	match mode {
		Mode::Find {
			weapon,
			opponent,
			min_score,
			allow_draw,
		} => {
			command::find(
				out,
				dict.get_weapon(&weapon.unwrap())?,
				actor::Actor::load(format!("data/patterns/{}.json", opponent.unwrap()), &dict)?,
				min_score.unwrap(),
				allow_draw,
			)?;
		}
		Mode::FindAll {
			weapon,
			input_dir,
			min_score,
			allow_draw,
		} => {
			command::find_all(out, &dict, dict.get_weapon(&weapon.unwrap())?, input_dir, min_score, allow_draw)?;
		}
		Mode::Consistents { weapon, input_dir, recursive } => {
			command::consistents(out, dict.get_weapon(&weapon.unwrap())?, input_dir.unwrap(), recursive)?;
		}
		Mode::Simulate { al_weapon: _, op_weapon: _ } => {
			todo!();
			// command::simulate(out, dict.get_weapon(&al_weapon.unwrap())?, dict.get_weapon(&op_weapon.unwrap())?)?;
		}
	}
	Ok(())
}
