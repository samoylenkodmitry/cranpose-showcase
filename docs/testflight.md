# Ship to TestFlight

The TestFlight workflow (`.github/workflows/testflight.yml`) builds the iPhone
app with the Apple Watch app inside it, signs both for the App Store and
uploads them to App Store Connect. People you invite then install the build
with Apple's TestFlight app. The workflow runs on a Mac of GitHub's, so you
need no Mac for it.

You do these steps once. After that, a new build is one click.

## 1. Join the Apple Developer Program

Enroll at <https://developer.apple.com/programs/enroll/> with your Apple
Account. It costs 99 USD a year. Apple can take a day or two to approve an
enrollment.

## 2. Register the bundle IDs

A bundle ID names an app across Apple's systems. It has to be yours, so pick
one under a domain you own or your name, such as `com.example.showcase`.

1. Open <https://developer.apple.com/account/resources/identifiers/list>.
2. Click **+**, choose **App IDs**, then **App**.
3. Description: `Showcase`. Bundle ID: **Explicit**, `com.example.showcase`.
   Click **Continue**, then **Register**.
4. For the watch app, do it again with the description `Showcase Watch` and
   the bundle ID `com.example.showcase.watchkitapp`: the iPhone app's ID with
   `.watchkitapp` after it. Skip this if you ship the iPhone app alone.

The template's own IDs are `io.cranpose.showcase` and
`io.cranpose.showcase.watchkitapp`. Step 7 tells the workflow to use yours. If
you would rather have your IDs in the code, change `CFBundleIdentifier` in
`ios/CranposeShowcase/Info.plist`, and `bundle-id` and `companion` under
`[package.metadata.cranpose.watchos]` in `Cargo.toml`.

## 3. Create the app in App Store Connect

1. Open <https://appstoreconnect.apple.com/apps> and click **+**, then
   **New App**.
2. Platform: **iOS**. Name: the name people see on the App Store; it has to
   be free. Primary language: yours. Bundle ID: the iPhone app's ID from
   step 2. SKU: any text, such as `showcase`.
3. Click **Create**.

The watch app needs no record of its own: it ships inside the iPhone app.

## 4. Make an App Store Connect API key

The workflow uploads with this key.

1. In App Store Connect, open **Users and Access**, then **Integrations**,
   then **App Store Connect API**, then **Team Keys**.
2. Click **+**. Name: `GitHub TestFlight`. Access: **App Manager**.
3. Note the **Issuer ID** above the list and the **Key ID** of the new key.
4. Click **Download** next to the key. You get `AuthKey_<KEY ID>.p8`. Apple
   lets you download it once, so keep it somewhere safe.

## 5. Make an Apple Distribution certificate

The certificate and its private key sign the app. GitHub takes them as one
`.p12` file. These commands work on macOS, Linux and Windows (Git Bash) with
OpenSSL:

```sh
openssl genrsa -out distribution.key 2048
openssl req -new -key distribution.key -out distribution.csr \
  -subj "/emailAddress=you@example.com/CN=Your Name/C=US"
```

1. Open <https://developer.apple.com/account/resources/certificates/list>
   and click **+**.
2. Choose **Apple Distribution**, click **Continue**, upload
   `distribution.csr` and download the certificate, `distribution.cer`.
3. Turn it into a `.p12` with a password of your choice:

```sh
openssl x509 -inform DER -in distribution.cer -out distribution.pem
openssl pkcs12 -export -legacy -inkey distribution.key -in distribution.pem \
  -out distribution.p12
```

macOS's own `openssl` (LibreSSL) has no `-legacy` option and needs none:
leave it out there.

On a Mac you can use Keychain Access instead: **Keychain Access > Certificate
Assistant > Request a Certificate From a Certificate Authority**, saved to
disk, gives the `.csr`. Double-click the downloaded `.cer`, then in **My
Certificates** right-click **Apple Distribution: Your Name** and choose
**Export** to save the `.p12`.

## 6. Make the App Store provisioning profiles

A profile ties a bundle ID to the certificate.

1. Open <https://developer.apple.com/account/resources/profiles/list> and
   click **+**.
2. Under **Distribution** choose **App Store Connect** and click
   **Continue**.
3. App ID: the iPhone app's. Certificate: the one from step 5. Name:
   `Showcase App Store`. Download the `.mobileprovision` file.
4. If you ship the watch app, make a second profile the same way for the
   watch app's ID, named `Showcase Watch App Store`.

## 7. Add the secrets to GitHub

In your repository on GitHub, open **Settings > Secrets and variables >
Actions**. Add each of these with **New repository secret**:

| Secret | Value |
|---|---|
| `IOS_DISTRIBUTION_CERTIFICATE` | `distribution.p12` in base64 |
| `IOS_DISTRIBUTION_CERTIFICATE_PASSWORD` | the password of `distribution.p12` |
| `IOS_APP_STORE_PROFILE` | the iPhone app's profile in base64 |
| `WATCH_APP_STORE_PROFILE` | the watch app's profile in base64; only when you ship the watch app |
| `APP_STORE_CONNECT_KEY_ID` | the Key ID from step 4 |
| `APP_STORE_CONNECT_ISSUER_ID` | the Issuer ID from step 4 |
| `APP_STORE_CONNECT_PRIVATE_KEY` | the whole text of `AuthKey_<KEY ID>.p8` |

Base64 of a file, copied to the clipboard on a Mac:

```sh
base64 -i distribution.p12 | pbcopy
```

On Linux, `base64 -w0 distribution.p12` prints it on one line.

If your bundle IDs are not the template's, open the **Variables** tab and add
the variable `IOS_BUNDLE_ID` with the iPhone app's ID, such as
`com.example.showcase`. The workflow gives the watch app the same ID with
`.watchkitapp` after it.

## 8. Run the workflow

1. Open the **Actions** tab, choose **TestFlight** and click **Run
   workflow**.
2. **Put the Apple Watch app in the upload**: leave it on to ship both apps,
   turn it off to ship the iPhone app alone.
3. **Build number**: leave it empty to use the run's number. Every upload of
   one version needs a higher build number than the one before.
4. Click **Run workflow**. It takes about half an hour. A missing secret
   stops it in its first step with the secret's name.

The version people see is `CFBundleShortVersionString` in
`ios/CranposeShowcase/Info.plist`. App Store Connect takes a few minutes to
process an upload and emails you when the build is ready.

## 9. Add testers

- **Your team** (up to 100 people with a role in App Store Connect): open
  the app in App Store Connect, then **TestFlight**. Next to **Internal
  Testing** click **+**, name the group and add the people. They get each
  processed build without review.
- **Anyone else** (up to 10,000 people): next to **External Testing** click
  **+**, add email addresses or turn on a public link, and add a build. Apple
  reviews the first build of each version before external testers get it,
  which takes about a day. Fill in **Test Information** first: a feedback
  email and what to test.

Testers install **TestFlight** from the App Store on the iPhone and accept
the invitation. The watch app installs from the iPhone's **Watch** app, or by
itself when the watch installs apps automatically.

## Upload from your own Mac

The workflow runs two scripts that also run on a Mac with Xcode 27 and Rust:

```sh
PROFILE=release WITH_WATCH=1 ./ios/build-app.sh aarch64-apple-ios
export IOS_DISTRIBUTION_CERTIFICATE="$(base64 -i distribution.p12)"
# ... the other variables from step 7
./ios/testflight.sh target/aarch64-apple-ios/release/CranposeShowcase.app 42 Showcase.ipa
```

`UPLOAD=0` makes the signed `.ipa` and uploads nothing.

## When it fails

- **"Missing: ..."**: add the secrets it names.
- **"The profile ... is for the bundle id ..."**: the profile belongs to
  another app ID. Make the profile for the ID the message names, or fix
  `IOS_BUNDLE_ID`.
- **"... is a development or ad hoc profile"**: make the profile under
  **Distribution > App Store Connect**.
- **"... holds no Apple Distribution certificate"**: the `.p12` is another
  kind of certificate, or it was exported without its private key.
- **The upload says the build number was used**: run it again with a higher
  build number.
- Certificates and profiles last a year. Make new ones and replace the
  secrets when they expire.
