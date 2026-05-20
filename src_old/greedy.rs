use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::error::Error;

pub fn greedy<P: AsRef<Path>>(dir_path: P, out_file: Option<P>) -> Result<(), Error> {
	let (opponent_matrix, all_opponents) = match load(dir_path)? {
		Some(result) => result,
		None => {
			println!("指定されたディレクトリにCSVファイルが見つかりませんでした。");
			return Ok(());
		}
	};
	println!("=== 読み込み完了: 対戦相手数 {}名 ===", all_opponents.len());

	let mut perfect_coverage: HashMap<String, HashSet<String>> = HashMap::new();
	for (opponent, records) in &opponent_matrix {
		for rec in records {
			perfect_coverage.entry(rec.clone()).or_insert_with(HashSet::new).insert(opponent.clone());
		}
	}

	let mut uncovered = all_opponents.clone();
	let mut selected_perfect_patterns: Vec<(String, Vec<String>)> = Vec::new();

	while !uncovered.is_empty() {
		let mut best_pattern: Option<String> = None;
		let mut best_covered_opponents: HashSet<String> = HashSet::new();
		let mut max_count = 0;

		for (pattern, covered_set) in &perfect_coverage {
			let current_covered: HashSet<String> = covered_set.intersection(&uncovered).cloned().collect();
			if current_covered.len() > max_count {
				max_count = current_covered.len();
				best_pattern = Some(pattern.clone());
				best_covered_opponents = current_covered;
			}
		}

		if max_count == 0 {
			break;
		}

		let chosen_pattern = best_pattern.unwrap();
		for op in &best_covered_opponents {
			uncovered.remove(op);
		}

		let mut covered_list: Vec<String> = best_covered_opponents.into_iter().collect();
		covered_list.sort();
		selected_perfect_patterns.push((chosen_pattern, covered_list));
	}

	if let Some(out_path) = out_file {
		output(out_path, &selected_perfect_patterns)?;
	}

	println!("必須パターン数: {} パターン", selected_perfect_patterns.len());
	for (idx, (pattern, targets)) in selected_perfect_patterns.iter().enumerate() {
		println!("\n[パターン {}] : {}", idx + 1, pattern);
		println!("  -> 100%勝てる相手 ({}名): {}", targets.len(), targets.join(", "));
	}

	Ok(())
}

fn load<P: AsRef<Path>>(dir_path: P) -> Result<Option<(HashMap<String, Vec<String>>, HashSet<String>)>, io::Error> {
	let mut opponent_matrix: HashMap<String, Vec<String>> = HashMap::new();
	let mut all_opponents = HashSet::new();

	let entries = fs::read_dir(dir_path)?;

	for entry in entries {
		let path = entry?.path();
		if path.is_file() {
			let opponent_name = path.file_stem().and_then(|s| s.to_str()).unwrap().to_string();
			all_opponents.insert(opponent_name.clone());

			let content = fs::read_to_string(&path)?;
			let mut records = Vec::new();

			for line in content.lines() {
				let line = line.trim();
				if line.is_empty() {
					continue;
				}
				let fields: Vec<&str> = line.split(',').collect();
				if fields.len() != 7 {
					continue;
				}
				let filtered_fields: String = fields.into_iter().skip(2).collect::<Vec<&str>>().join(",");
				records.push(filtered_fields);
			}
			opponent_matrix.insert(opponent_name, records);
		}
	}

	if all_opponents.is_empty() {
		return Ok(None);
	}

	Ok(Some((opponent_matrix, all_opponents)))
}

fn output<P: AsRef<Path>>(out_path: P, patterns: &[(String, Vec<String>)]) -> Result<(), io::Error> {
	let mut file = fs::File::create(out_path)?;
	for (pattern, targets) in patterns {
		writeln!(file, "{}", pattern)?;
		writeln!(file, "  -> 100%勝てる相手 ({}名): {}", targets.len(), targets.join(", "))?;
	}
	Ok(())
}
