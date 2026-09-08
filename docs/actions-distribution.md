# OpenSCQ30 Actions distribution

[Add OpenSCQ30 Actions to Obtainium](https://apps.obtainium.imranr.dev/redirect?r=obtainium://app/%7B%22id%22%3A%22com.oppzippy.openscq30.actions%22%2C%22url%22%3A%22https%3A%2F%2Fgithub.com%2Fcolonelpanic8%2FOpenSCQ30%22%2C%22author%22%3A%22colonelpanic8%22%2C%22name%22%3A%22OpenSCQ30%20Actions%22%2C%22additionalSettings%22%3A%22%7B%5C%22includePrereleases%5C%22%3Atrue%2C%5C%22apkFilterRegEx%5C%22%3A%5C%22%5Eopenscq30-actions-arm64-v8a%5C%5C%5C%5C.apk%24%5C%22%7D%22%7D).

The fork publishes arm64 Android APKs as GitHub prereleases. Obtainium can track
`https://github.com/colonelpanic8/OpenSCQ30` with **Include prereleases** enabled
and the APK filter `openscq30-actions-arm64-v8a\.apk$`.

This channel uses package `com.oppzippy.openscq30.actions`, the app name
**OpenSCQ30 Actions**, and a dedicated signing key. It can coexist with upstream
OpenSCQ30. Every update must use the same signing key and a higher Android
`versionCode`; the tag should match `versionName` with a leading `v`.

To build, provide `OPENSCQ30_KEYSTORE_PATH` and `OPENSCQ30_KEYSTORE_PASSWORD` in the
environment, using keystore alias `openscq30-actions`, and run:

```sh
cd android
./gradlew assembleArm64-v8aActions
```

The Actions build disables Android debugging and uses the unoptimized Rust
profile while the device support is experimental. Never publish an unsigned
APK. Verify its signature and package ID before attaching
`app/build/outputs/apk/arm64-v8a/actions/app-arm64-v8a-actions.apk` to a release as
`openscq30-actions-arm64-v8a.apk`.

The first release includes experimental read-only Liberty 5 Pro support and
custom assistant actions. Physical gesture delivery remains unverified; see
[Liberty 5 Pro](liberty-5-pro.md) and [custom actions](custom-actions.md).
