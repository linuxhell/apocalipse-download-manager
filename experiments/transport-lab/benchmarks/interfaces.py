"""Alternating physical-interface QUIC Link experiment; requires a separate server.

The server must run `serve-many` for rounds*3 transfers. Token is inherited from
ADM_LINK_LAB_TOKEN and is never put in argv or the report. No interface is disabled
by this script. To measure failure, disconnect a secondary network manually.
"""
import argparse, hashlib, json, pathlib, platform, statistics, subprocess, time
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--lab', type=pathlib.Path, required=True)
p.add_argument('--remote', required=True)
p.add_argument('--certificate', type=pathlib.Path, required=True)
p.add_argument('--primary-ip', required=True)
p.add_argument('--secondary-ip', required=True)
p.add_argument('--rounds', type=int, default=3)
p.add_argument('--output', type=pathlib.Path, required=True)
p.add_argument('--networks', required=True, help='Describe physical adapters and independent gateways, without credentials')
a = p.parse_args()
if not 1 <= a.rounds <= 20 or a.primary_ip == a.secondary_ip: p.error('Use 1..20 rounds and two different assigned IP addresses')
a.output.mkdir(parents=True, exist_ok=True)
rows = []
profiles = [('primary', a.primary_ip, []), ('secondary', a.secondary_ip, []), ('multipath', a.primary_ip, [a.secondary_ip])]
for round_ in range(a.rounds):
    for index in range(3):
        label, bind, extras = profiles[(round_ + index) % 3]
        out = a.output / f'{round_}-{label}.bin'
        command = [str(a.lab.resolve()), 'get-bound', a.remote, str(a.certificate.resolve()), str(out), bind, *extras]
        started = time.monotonic()
        done = subprocess.run(command, check=True, capture_output=True, text=True, timeout=650)
        row = json.loads(done.stdout.strip()); row.update(profile=label, round=round_, wall_seconds=time.monotonic()-started)
        with out.open('rb') as f: digest = hashlib.file_digest(f, 'sha256').hexdigest()
        if digest != row['sha256'] or out.stat().st_size != row['bytes']: raise RuntimeError('Output hash/length mismatch')
        rows.append(row); out.unlink()
        print(json.dumps(row), flush=True)
        report = dict(platform=platform.platform(), networks=a.networks, caveat='UDP path counters include overhead. Different local IPs alone do not prove independent WAN links.', samples=rows)
        (a.output / 'report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
if len({r['sha256'] for r in rows}) != 1: raise RuntimeError('The server file changed between profiles')
print(json.dumps({label: statistics.median(r['wall_seconds'] for r in rows if r['profile'] == label) for label, _, _ in profiles}))
