use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::error::Error;

pub fn greedy<P: AsRef<Path>>(dir_path: P) -> Result<(), Error> {
	// 1. 全CSVファイルを走査してデータを構造化
	// opponent_name -> Vec<PatternRecord>
	let mut opponent_matrix: HashMap<String, Vec<String>> = HashMap::new();
	let mut all_opponents = HashSet::new();

	let entries = fs::read_dir(dir_path)?;

	for entry in entries {
		let path = entry?.path();
		if path.is_file() {
			// ファイル名を対戦相手の識別子（enoや名前）として取得
			let opponent_name = path.file_stem().and_then(|s| s.to_str()).unwrap().to_string();
			all_opponents.insert(opponent_name.clone());

			let content = fs::read_to_string(&path)?;
			let mut records = Vec::new();

			for line in content.lines() {
				let line = line.trim();
				if line.is_empty() {
					continue;
				}
				records.push(line.to_string());
			}
			opponent_matrix.insert(opponent_name, records);
		}
	}

	if all_opponents.is_empty() {
		println!("指定されたディレクトリにCSVファイルが見つかりませんでした。");
		return Ok(());
	}
	println!("=== 読み込み完了: 対戦相手数 {}名 ===", all_opponents.len());

	// 2. 逆引きマップの作成: 各パターンが「どの相手に100%勝てるか」
	// 自分のパターン文字列 -> HashSet<対戦相手名>
	let mut perfect_coverage: HashMap<String, HashSet<String>> = HashMap::new();
	for (opponent, records) in &opponent_matrix {
		for rec in records {
			perfect_coverage.entry(rec.clone()).or_insert_with(HashSet::new).insert(opponent.clone());
		}
	}

	// 3. 【第1フェーズ】貪欲法による100%勝てる最小セットの選出
	let mut uncovered = all_opponents.clone();
	let mut selected_perfect_patterns: Vec<(String, Vec<String>)> = Vec::new();

	while !uncovered.is_empty() {
		let mut best_pattern: Option<String> = None;
		let mut best_covered_opponents: HashSet<String> = HashSet::new();
		let mut max_count = 0;

		// まだカバーしていない相手を「最も多く巻き込んで100%撃破できる」パターンを探索
		for (pattern, covered_set) in &perfect_coverage {
			let current_covered: HashSet<String> = covered_set.intersection(&uncovered).cloned().collect();
			if current_covered.len() > max_count {
				max_count = current_covered.len();
				best_pattern = Some(pattern.clone());
				best_covered_opponents = current_covered;
			}
		}

		// 新たに100%勝てる相手を増やせるパターンがもう存在しない（全カバー完了、または相性限界）
		if max_count == 0 {
			break;
		}

		let chosen_pattern = best_pattern.unwrap();
		// カバーした相手を未カバーリストから削除
		for op in &best_covered_opponents {
			uncovered.remove(op);
		}

		let mut covered_list: Vec<String> = best_covered_opponents.into_iter().collect();
		covered_list.sort(); // 見やすくソート
		selected_perfect_patterns.push((chosen_pattern, covered_list));
	}

	// 4. 結果表示（100%勝利セット）
	println!("必須パターン数: {} パターン", selected_perfect_patterns.len());
	for (idx, (pattern, targets)) in selected_perfect_patterns.iter().enumerate() {
		println!("\n[パターン {}] : {}", idx + 1, pattern);
		println!("  -> 100%勝てる相手 ({}名): {}", targets.len(), targets.join(", "));
	}

	Ok(())
}
