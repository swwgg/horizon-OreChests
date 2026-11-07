#!/usr/bin/env python3
"""Package the mod as a source-only `.hnmod` (and optionally `.hmod`), the files Horizon takes.

The `.hnmod` archive holds `horizon-native.json` and `mixins/<id>/` (only `.rs`, `.toml` and `data/`),
with fixed dates and modes, so the same sources always give the same bytes and the same SHA-256.
The launcher compiles it on the pinned Horizon revision.
"""
import hashlib
import json
from pathlib import Path
import re
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
GAME = '26.3'


def package():
    compat = json.loads((ROOT / 'compatibility.json').read_text())
    manifests = sorted((ROOT / 'mixins').glob('*/horizon-mixin.toml'))
    if len(manifests) != 1:
        raise SystemExit('a .hnmod holds one bundle: keep one mixins/<id>/horizon-mixin.toml')
    bundle = manifests[0].parent
    mixin = tomllib.loads(manifests[0].read_text())
    mod = mixin['mod']
    if bundle.name != mod['id']:
        raise SystemExit(f"rename mixins/{bundle.name}/ to mixins/{mod['id']}/ (the registry finds it by id)")
    if not mod['id'].replace('_', '').isalnum() or not mod['id'].islower():
        raise SystemExit('the registry takes ids of lowercase letters, digits and _ only')
    if mixin.get('compatibility', {}).get('horizon_revision') != compat['horizon_revision']:
        raise SystemExit('horizon-mixin.toml and compatibility.json pin different Horizon revisions')

    manifest = dict(schema=1, id=mod['id'], version=mod['version'], game=GAME,
                    horizon=compat['horizon_revision'], mixins=compat['horizon_mixins_revision'])

    # 1. Build .hmod containing content (assets/data) + horizon-mod.toml
    hmod_out = ROOT / 'target' / f"{mod['id']}-{mod['version']}.hmod"
    hmod_out.parent.mkdir(parents=True, exist_ok=True)
    if (ROOT / 'assets').exists() and (ROOT / 'horizon-mod.toml').exists():
        hmod_files = {'horizon-mod.toml': (ROOT / 'horizon-mod.toml').read_bytes()}
        for folder in ['assets', 'data']:
            f_path = ROOT / folder
            if f_path.exists():
                for p in sorted(f_path.rglob('*')):
                    if p.is_file():
                        hmod_files[p.relative_to(ROOT).as_posix()] = p.read_bytes()
        with zipfile.ZipFile(hmod_out, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
            for name, content in sorted(hmod_files.items()):
                info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o100644 << 16
                archive.writestr(info, content)
        print(f"Created content .hmod: {hmod_out} ({hmod_out.stat().st_size} bytes)")

        # 2. Embed .hmod bytes into orechests_glue.rs
        glue_file = bundle / 'orechests_glue.rs'
        if glue_file.exists():
            glue_text = glue_file.read_text()
            b = hmod_out.read_bytes()
            escaped = ''.join(f'\\x{byte:02x}' for byte in b)
            start_marker = 'pub const HMOD_BYTES: &[u8] = b"'
            end_marker = '";\n\npub fn ensure_content_mod_installed'
            if start_marker in glue_text and end_marker in glue_text:
                s_idx = glue_text.index(start_marker) + len(start_marker)
                e_idx = glue_text.index(end_marker, s_idx)
                new_glue = glue_text[:s_idx] + escaped + glue_text[e_idx:]
                glue_file.write_text(new_glue)
                print(f"Embedded {len(b)} bytes of .hmod into {glue_file.name}")

    # 3. Collect files inside the bundle for .hnmod
    files = {'horizon-native.json': (json.dumps(manifest, indent=2, sort_keys=True) + '\n').encode()}
    for path in sorted(bundle.rglob('*')):
        if path.is_file():
            if path.suffix in ('.rs', '.toml') or 'data' in path.parts:
                files[path.relative_to(ROOT).as_posix()] = path.read_bytes()

    out = ROOT / 'target' / f"{mod['id']}-{mod['version']}.hnmod"
    with zipfile.ZipFile(out, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)

    print(json.dumps(dict(file=str(out), size=out.stat().st_size,
                          sha256=hashlib.sha256(out.read_bytes()).hexdigest()), indent=2))


if __name__ == '__main__':
    package()
