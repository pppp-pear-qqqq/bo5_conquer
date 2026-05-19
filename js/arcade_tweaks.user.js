// ==UserScript==
// @name         ARCADE Tweaks
// @namespace    http://tampermonkey.net/
// @version      0.1.0
// @description  アーケードモード攻略時に発生するいろんな不満の解消
// @author       なしなし
// @match        https://wdrb.work/bo5/battle_lobby.php?mode=arcade
// @icon         https://www.google.com/s2/favicons?sz=64&domain=wdrb.work/bo5
// @grant        none
// ==/UserScript==

(function () {
	'use strict';
	function bake(tagName, f) {
		const e = document.createElement(tagName);
		f(e);
		return e;
	}
	const form = document.getElementById('btlb_form_arcade');

	// ページ遷移省略
	{
		const key = 'prevent_arcade_move';

		const options = document.querySelector('.setup_link');
		options.parentElement.insertBefore(bake('label', e => {
			e.appendChild(bake('input', e => {
				e.type = 'checkbox';
				e.checked = localStorage.getItem(key) === 'true';
				e.addEventListener('change', () => localStorage.setItem(key, e.checked ? 'true' : 'false'));
			}));
			e.appendChild(document.createTextNode('戦闘結果の表示をスキップ'));
		}), options);

		form.addEventListener('submit', e => {
			if (localStorage.getItem(key) !== 'true') return;
			e.preventDefault();
			fetch(form.action, { method: form.method, body: new FormData(form) }).then(r => {
				if (r.ok) {
					location.reload();
				} else {
					console.error('サーバーエラーが発生しました。');
					alert('サーバーエラーが発生しました。');
				}
			}).catch(e => {
				console.error('通信エラー:', e);
			});
		});
	}

	// 選択時名前で自動検索
	{
		const search_box = document.getElementById('drilldown');
		for (const e of form.querySelector('.battle_npc').children) {
			e.addEventListener('click', () => {
				const stage = e.dataset.stage;
				const name = form.querySelector(`.next_ch[data-npc="${stage}"] [data-name]`).dataset.name;
				search_box.value = `${stage}|${name}`;
				search_box.dispatchEvent(new Event('input'));
			})
		}
	}
})();
