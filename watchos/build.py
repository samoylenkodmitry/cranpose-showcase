#!/usr/bin/env python3
"""Builds the Apple Watch app.

The watch app is `watch::WatchShowcase`, the composable that Cargo.toml names
under `[package.metadata.cranpose.watchos]`. Cranpose ships the watchOS host
(SwiftUI, SpriteKit and a generated Rust runner) inside the `cranpose` crate.
This script finds the version that Cargo.lock pins and runs its packager, so
the repository holds no Swift code and no Xcode project.

    python3 watchos/build.py                     # app for the watch simulator
    python3 watchos/build.py --smoke             # and launch it in a new simulator
    python3 watchos/build.py --target device     # unsigned app for arm64 watches

The app is written to target/watchos/<target>/CranposeWatch.app. Other
options, such as --deployment-target or --sign IDENTITY, go to the packager.
It needs macOS with Xcode, the watchOS SDK and Python 3.11 or newer.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def cranpose_dir():
    """The directory of the `cranpose` crate that Cargo.lock pins."""
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--locked', '--format-version', '1'], cwd=ROOT, text=True))
    package = next(p for p in metadata['packages'] if p['name'] == 'cranpose')
    print(f"Cranpose {package['version']}", flush=True)
    return Path(package['manifest_path']).parent


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--target', choices=['simulator', 'device'], default='simulator')
    parser.add_argument('--smoke', action='store_true',
                        help='install the simulator app in a new watch simulator, launch it, '
                             'and keep a screenshot in target/watchos/simulator/evidence')
    args, packager_args = parser.parse_known_args()
    if args.smoke and args.target != 'simulator':
        parser.error('--smoke needs --target simulator')
    if sys.version_info < (3, 11):
        parser.error('the watchOS packager needs Python 3.11 or newer')
    if (args.target == 'device' and sys.platform == 'darwin'
            and not any(arg.startswith('--deployment-target') for arg in packager_args)):
        # The packager builds a device app for watchOS 27, the first version
        # for which the App Store takes an arm64-only watch app. An older
        # Xcode has no watchOS 27 SDK, so the app is built for the SDK's own
        # version: it installs on a watch, and the App Store refuses it.
        sdk = subprocess.check_output(['xcrun', '--sdk', 'watchos', '--show-sdk-version'],
                                      text=True).strip()
        major = int(sdk.split('.')[0])
        if major < 27:
            print(f'watchOS SDK {sdk}: building for watchOS {major}.0; '
                  'the App Store needs Xcode 27 or newer', flush=True)
            packager_args += ['--deployment-target', f'{major}.0']
    host = cranpose_dir() / 'watchos'
    output = ROOT / 'target/watchos' / args.target
    subprocess.run([sys.executable, host / 'build.py', '--manifest', ROOT / 'Cargo.toml',
                    '--target', args.target, '--output', output, *packager_args], check=True)
    if args.smoke:
        subprocess.run([sys.executable, host / 'smoke.py', output / 'CranposeWatch.app',
                        output / 'evidence'], check=True)


if __name__ == '__main__':
    main()
