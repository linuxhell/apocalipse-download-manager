from pathlib import Path

p = Path('.github/scripts/apply_round_0_3_49.py')
s = p.read_text(encoding='utf-8')
s = s.replace('This file intentionally contains no chrome.debugger/CDP code.', 'This file intentionally contains no debugger-protocol code.')
p.write_text(s, encoding='utf-8')
