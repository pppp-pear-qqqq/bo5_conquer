// ==UserScript==
// @name         アーケード表示簡易化
// @namespace    http://tampermonkey.net/
// @version      2026-05-15
// @description  アーケードモードで、戦闘ログ画面への遷移をキャンセルする
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

	const form = document.getElementById('btlb_form_arcade');
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
})();
