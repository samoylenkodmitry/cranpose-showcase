#!/usr/bin/env python3
"""Signs the iPhone app, and the Apple Watch app in it, for the App Store and
packs them into an .ipa for TestFlight.

Build the app first:

    PROFILE=release WITH_WATCH=1 ios/build-app.sh aarch64-apple-ios

then:

    python3 ios/ipa.py target/aarch64-apple-ios/release/CranposeShowcase.app \\
        --build 42 --identity "Apple Distribution" \\
        --app-profile app.mobileprovision --watch-profile watch.mobileprovision \\
        --output Showcase.ipa

--bundle-id gives the iPhone app another bundle id; the watch app's is the
same with `.watchkitapp` after it. Without --watch-profile the app must have
no watch app in it (build it without WITH_WATCH). --ad-hoc signs with an ad
hoc signature and no profiles: it checks the packaging on a computer with no
signing material, and App Store Connect refuses what it makes.

The app bundle is left as it was; the script works on a copy. The TestFlight
workflow (.github/workflows/testflight.yml) runs this script, and
docs/testflight.md says where each input comes from.
"""
import argparse
import datetime
import plistlib
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

PRIVACY = Path(__file__).resolve().parent / 'PrivacyInfo.xcprivacy'
WATCH = Path('Watch/CranposeWatch.app')
WATCH_SUFFIX = '.watchkitapp'


def output(*args):
    return subprocess.check_output(args, text=True).strip()


def run(*args):
    print('+', ' '.join(map(str, args)), flush=True)
    subprocess.run(list(map(str, args)), check=True)


def read_plist(path):
    with path.open('rb') as file:
        return plistlib.load(file)


def write_plist(path, value):
    with path.open('wb') as file:
        plistlib.dump(value, file)


def platform_keys(sdk, platform):
    """What an Xcode build for `sdk` writes into Info.plist. App Store Connect
    reads them to tell which SDK and Xcode made the build."""
    version = output('xcrun', '--sdk', sdk, '--show-sdk-version')
    build = output('xcrun', '--sdk', sdk, '--show-sdk-build-version')
    xcode = output('xcodebuild', '-version').splitlines()
    parts = (re.match(r'Xcode (\S+)', xcode[0]).group(1).split('.') + ['0', '0'])[:3]
    return {
        'BuildMachineOSBuild': output('sw_vers', '-buildVersion'),
        'CFBundleSupportedPlatforms': [platform],
        'DTCompiler': 'com.apple.compilers.llvm.clang.1_0',
        'DTPlatformBuild': build,
        'DTPlatformName': sdk,
        'DTPlatformVersion': version,
        'DTSDKBuild': build,
        'DTSDKName': f'{sdk}{version}',
        'DTXcode': f'{int(parts[0]):02d}{int(parts[1])}{int(parts[2])}',
        'DTXcodeBuild': xcode[1].split()[-1],
    }


def decode_profile(path):
    return plistlib.loads(subprocess.check_output(['security', 'cms', '-D', '-i', str(path)]))


def check_profile(profile, path, bundle_id):
    """Stops with a plain message when `profile` cannot sign `bundle_id` for
    the App Store."""
    name = profile.get('Name', path.name)
    app_id = profile.get('Entitlements', {}).get('application-identifier', '')
    team = (profile.get('TeamIdentifier') or [''])[0]
    granted = app_id.removeprefix(f'{team}.')
    if granted != bundle_id and not (granted.endswith('*') and bundle_id.startswith(granted[:-1])):
        sys.exit(f'The profile "{name}" ({path}) is for the bundle id {granted}, '
                 f'and the app is {bundle_id}. Make a profile for {bundle_id}.')
    if profile.get('ProvisionedDevices') or profile.get('Entitlements', {}).get('get-task-allow'):
        sys.exit(f'The profile "{name}" ({path}) is a development or ad hoc profile. '
                 'TestFlight takes an App Store profile (Distribution: App Store Connect).')
    expires = profile.get('ExpirationDate')
    if expires and expires.replace(tzinfo=datetime.timezone.utc) < datetime.datetime.now(datetime.timezone.utc):
        sys.exit(f'The profile "{name}" ({path}) expired on {expires:%Y-%m-%d}. Make a new one.')


def entitlements(profile, bundle_id):
    """The profile's entitlements for `bundle_id`. A wildcard profile's `*`
    becomes the bundle id, as Xcode does it."""

    def resolve(value):
        if isinstance(value, str) and value.endswith('.*'):
            return value[:-1] + bundle_id
        if isinstance(value, list):
            return [resolve(item) for item in value]
        return value

    return {key: resolve(value) for key, value in profile['Entitlements'].items()}


def prepare(bundle, info_changes, sdk, platform, profile_path, args, work):
    """Writes the bundle's Info.plist, privacy manifest and profile, then signs
    it."""
    info = read_plist(bundle / 'Info.plist')
    info.update(platform_keys(sdk, platform))
    info.update(info_changes)
    write_plist(bundle / 'Info.plist', info)
    shutil.copyfile(PRIVACY, bundle / PRIVACY.name)
    if args.ad_hoc:
        run('codesign', '--force', '--sign', '-', bundle)
        return
    profile = decode_profile(profile_path)
    check_profile(profile, profile_path, info['CFBundleIdentifier'])
    shutil.copyfile(profile_path, bundle / 'embedded.mobileprovision')
    entitlements_path = work / f'{bundle.stem}.entitlements'
    write_plist(entitlements_path, entitlements(profile, info['CFBundleIdentifier']))
    keychain = ['--keychain', args.keychain] if args.keychain else []
    run('codesign', '--force', '--sign', args.identity, '--entitlements', entitlements_path,
        '--generate-entitlement-der', '--timestamp', *keychain, bundle)


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('app', type=Path, help='the CranposeShowcase.app that ios/build-app.sh built')
    parser.add_argument('--build', required=True,
                        help='the build number, which grows with every upload of a version')
    parser.add_argument('--version', help="the version; the Info.plist's by default")
    parser.add_argument('--bundle-id', help="the iPhone app's bundle id; the Info.plist's by default")
    parser.add_argument('--identity', help='SHA-1 or name of the Apple Distribution identity')
    parser.add_argument('--app-profile', type=Path, help="the iPhone app's App Store profile")
    parser.add_argument('--watch-profile', type=Path, help="the watch app's App Store profile")
    parser.add_argument('--keychain', help='the keychain that holds the identity')
    parser.add_argument('--ad-hoc', action='store_true',
                        help='sign ad hoc with no profiles, to check the packaging')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()

    if not args.build.isdigit():
        parser.error('the build number is a whole number, like 42')
    if not args.ad_hoc and not (args.identity and args.app_profile):
        parser.error('--identity and --app-profile are needed unless --ad-hoc is given')
    if not (args.app / 'Info.plist').is_file():
        parser.error(f'{args.app} is not an app bundle; build it with ios/build-app.sh')
    has_watch = (args.app / WATCH).is_dir()
    if not args.ad_hoc and has_watch and not args.watch_profile:
        parser.error(f'{args.app} holds a watch app, so --watch-profile is needed; '
                     'or build the app without WITH_WATCH=1')
    if args.watch_profile and not has_watch:
        parser.error(f'--watch-profile was given, and {args.app} holds no watch app; '
                     'build it with WITH_WATCH=1')

    info = read_plist(args.app / 'Info.plist')
    bundle_id = args.bundle_id or info['CFBundleIdentifier']
    version = args.version or info['CFBundleShortVersionString']
    if not re.fullmatch(r'[A-Za-z0-9]+([.-][A-Za-z0-9]+)+', bundle_id):
        parser.error(f'{bundle_id} is not a bundle id')
    if not re.fullmatch(r'\d+(\.\d+){0,2}', version):
        parser.error(f'the version is like 1.2.3, and {version} is not')
    numbers = {
        'CFBundleShortVersionString': version,
        'CFBundleVersion': args.build,
    }

    with tempfile.TemporaryDirectory() as temporary:
        work = Path(temporary)
        payload = work / 'Payload'
        app = payload / args.app.name
        payload.mkdir()
        shutil.copytree(args.app, app, symlinks=True)
        # Sealed inside out: the iPhone app's signature covers the watch app.
        if has_watch:
            prepare(app / WATCH, {
                **numbers,
                'CFBundleIdentifier': bundle_id + WATCH_SUFFIX,
                'WKCompanionAppBundleIdentifier': bundle_id,
            }, 'watchos', 'WatchOS', args.watch_profile, args, work)
        prepare(app, {**numbers, 'CFBundleIdentifier': bundle_id},
                'iphoneos', 'iPhoneOS', args.app_profile, args, work)
        run('codesign', '--verify', '--deep', '--strict', app)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.unlink(missing_ok=True)
        # Without extended attributes: ditto would keep them as AppleDouble
        # `._` files beside each file, which break the watch app's signature.
        run('ditto', '-c', '-k', '--norsrc', '--noextattr', '--noqtn', '--noacl', '--keepParent',
            payload, args.output)
    with zipfile.ZipFile(args.output) as archive:
        stray = [name for name in archive.namelist() if Path(name).name.startswith('._')]
    if stray:
        sys.exit(f'{args.output} holds AppleDouble files: {stray[:3]}')
    print(f'{args.output}: {bundle_id} {version} ({args.build})'
          + (', with the watch app' if has_watch else ''))


if __name__ == '__main__':
    main()
