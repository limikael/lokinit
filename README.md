# Lokinit

> A fork of miniquad focused on windowing.

[![Discord](https://img.shields.io/discord/1092477105079595038?color=7289DA&label=%20&logo=discord&logoColor=white)](https://discord.gg/D5pzrmyqz3)

Miniquad is an amazing project. It has next to zero dependencies - only system ones like `libc` on Linux, `objc` on MacOS and `winapi` on Windows. The total amount of dependencies across all platforms and features is a crushing **7**:

```
$ cargo tree --target all
miniquad v0.3.15 (/home/speykious/repos/github.com/not-fl3/miniquad)
├── libc v0.2.139
├── ndk-sys v0.2.2
├── objc v0.2.7
│   └── malloc_buf v0.0.6
│       └── libc v0.2.139
└── winapi v0.3.9
    ├── winapi-i686-pc-windows-gnu v0.4.0
    └── winapi-x86_64-pc-windows-gnu v0.4.0
```

Cargo may be an amazing package manager, but making projects with hundreds of dependencies nested in a deep tree has some disadvantages:
- it clogs up compile times by orders of magnitude. Upon a `cargo clean`, it's very easy to see middle-sized projects take several minutes to build. Miniquad on the other hand takes *less than 5 seconds* in the same situation.
- we get restricted to what the dependencies can let us do with their goal. The more dependencies we use, the more we'll have to rely on other developers to fix problems affect us.
- quite often, the code of such dependencies is much more complex than it needs to be for our use-case, probably as a result of trying to be broad enough for lots of other use-cases.

On the other hand, rolling with our own code may also have its own disadvantages:
- the burden is placed on the developer to maintain code that they have to write themselves.
- there is no guarantee that the code written will turn out to be a better solution than one that is battle-tested or simply more mature.

In short, there definitely is a balance to be had when it comes to dependencies. But I believe that balance should be _heavily_ shifted to a place where 100 dependencies is extravagant or niche rather than the norm.

When it comes to native window management, the task at hand is to interact with the relevant system libraries on each operating system to spawn a window on the screen, and manage all the various properties that a window might need: resizing, fullscreen, having a title, binding to a graphics API so we can draw stuff to the screen. We shouldn't need any other dependencies than system ones, and miniquad has shown that.

However, Miniquad is *not* a windowing project, it is a cross-platform graphics rendering project that leverages a lot of custom-made code for windowing. The one and most advanced Rust project when it comes to windowing is [Winit](https://crates.io/crates/winit). It seems to work extremely well so far and has [a lot of features under its belt](https://github.com/rust-windowing/winit/blob/master/FEATURES.md#windowing-1) for each OS they support. The problem is that it pulls out 60 dependencies for this in a deep nested tree of dependencies, just on one OS. It takes several minutes to compile after a `cargo clean`. To make matters worse, if you need OpenGL, you need to add `glutin`, `glutin-winit` and `raw-window-handle` as hard dependencies too. I am convinced that we can do better than this, and have something that weighs almost nothing in terms of dependencies and compile times. That way, projects using Lokinit won't have to pull in 60 dependencies out of the box.

# Status of Lokinit

As Lokinit was forked first and foremost to help in the development of [Loki](https://github.com/loki-chat), our chat app written from scratch entirely in Rust, it is currently very mildly modified from Miniquad 0.4.

It also has a hard dependency on `skia-safe` for fast 2D drawing, as the quickest way to get Skia working on all 5 platforms was to code it directly into the fork. In the future though, the plan is to remove this hard dependency entirely so that lokinit can be solely a windowing library that may also provide various graphics API initialization endpoints.

## Supported Platforms

We intend to support the 5 major native platforms:

- Windows
- Linux
- Android
- MacOS
- iOS

We don't intend to support Web as a target. The web has vastly different needs and constraints than native systems when it comes to windowing, so if there ever is a need for it, it should probably be in a different crate with a completely different API.

# Building examples

## Linux

```bash
cargo run --example skia
```

On NixOS Linux you can use [`shell.nix`](shell.nix) to start a development
environment where Lokinit can be built and run.

## Windows

```bash
# both MSVC and GNU target is supported:
rustup target add x86_64-pc-windows-msvc
# or
rustup target add x86_64-pc-windows-gnu

cargo run --example skia
```

## Android

The recommended way to build for Android is using Docker.

Lokinit uses a fork of `cargo-quad-apk` called `cargo-loki-apk`, a slightly modifed version of `cargo-apk` specifically for Lokinit projects.

```
docker run --rm -v $(pwd)":/root/src" -w /root/src loki-chat/cargo-loki-apk cargo loki-apk build --example skia
```

The APK file will be in `target/android-artifacts/(debug|release)/apk`.

With the `log-impl` feature enabled, all log calls will be forwarded to the `adb` console.
No code modification for Android is required, everything should just work.

## iOS

To run on the simulator:

```
mkdir MyGame.app
cargo build --target x86_64-apple-ios --release
cp target/release/mygame MyGame.app
# only if the game have any assets
cp -r assets MyGame.app
cat > MyGame.app/Info.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
<key>CFBundleExecutable</key>
<string>mygame</string>
<key>CFBundleIdentifier</key>
<string>com.mygame</string>
<key>CFBundleName</key>
<string>mygame</string>
<key>CFBundleVersion</key>
<string>1</string>
<key>CFBundleShortVersionString</key>
<string>1.0</string>
</dict>
</plist>
EOF

xcrun simctl install booted MyGame.app/
xcrun simctl launch booted com.mygame
```

For details and instructions on provisioning for a real iphone, check [https://macroquad.rs/articles/ios/](https://macroquad.rs/articles/ios/).

## Cross Compilation

> **Note:** cross-compilation is now harder due to the restrictions that Skia has put in terms of environment setup. This should all go away once Lokinit doesn't depend on Skia anymore.

```bash

# windows target from linux host:
# this is how windows builds are tested from linux machine:
rustup target add x86_64-pc-windows-gnu
cargo run --example quad --target x86_64-pc-windows-gnu
```

# Goals of Lokinit

- **Fast compilation times.** Lokinit should compile under 5 seconds just like Miniquad.
- **Minimal dependencies.** We shouldn't need more than system library dependencies to do windowing properly. For such a task, 10 dependencies total is already considered too much.
- **Cross-platform.** It should support all 5 major native platforms as flawlessly as possible, potentially more OSes in the future like Redox.
- **Focus on windowing.** Our feature goals are pretty much the exact same as Winit, to spawn a window on all native platforms. People should be able to easily use Lokinit as a dependency and bind whatever graphics API they want: OpenGL, Vulkan, DirectX, Metal, etc. without needing to fork it and modify its internals for it to work.

# License

Lokinit is dual-licensed under APACHE and MIT.
See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
