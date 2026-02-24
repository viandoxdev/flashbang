# Flashbang

Flashbang is a modern, open-source flashcard application for Android that leverages the power of Rust for its core logic and Jetpack Compose for a beautiful, responsive UI.

## Features

- **Typst-based Cards:** Render rich text, mathematical equations, and complex layouts using the Typst typesetting system.
- **FSRS Scheduler:** Implements the Free Spaced Repetition Scheduler (FSRS) algorithm to optimize your learning efficiency.
- **GitHub Sync:** Load and sync your card decks directly from GitHub repositories.
- **Fuzzy Search:** Quickly find specific cards or concepts with powerful fuzzy search capabilities.
- **Statistics & Visualization:** Track your progress with detailed charts and statistics powered by Vico.
- **Material 3 Design:** A modern, intuitive interface built with Jetpack Compose and Material 3.
- **Offline First:** Your data is stored locally for fast access anytime, anywhere.

## Architecture

Flashbang follows a hybrid architecture to combine performance with modern Android development practices:

- **Frontend (Android):** Written in Kotlin using Jetpack Compose for UI, Hilt for dependency injection, and Navigation 3 for navigation.
- **Backend (Rust):** The core logic (scheduler, card parsing, search, GitHub sync) resides in the `fb-core` Rust crate.
- **Communication:** [UniFFI](https://github.com/mozilla/uniffi-rs) bridges the gap, generating type-safe Kotlin bindings for the Rust library.

## Building

### Prerequisites

Ensure you have the following installed:

- **Rust Toolchain:** Install Rust via [rustup](https://rustup.rs/).
- **Android Studio:** With the Android SDK and NDK installed.
- **cargo-ndk:** Install with `cargo install cargo-ndk`.
- **uniffi-bindgen:** Install with `cargo install uniffi-bindgen`.

You will also need to add the Android targets to your Rust installation:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

### Steps

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/flashbang.git
    cd flashbang
    ```

2.  **Build the Rust core and generate bindings:**
    Run the build script from the `fb-core` directory:
    ```bash
    cd fb-core
    ./build-android.sh
    cd ..
    ```
    This script compiles the Rust code for Android architectures and generates the necessary Kotlin bindings in `app/src/main/java/dev/vndx/flashbang/rust`.

3.  **Run the Android App:**
    Open the project in Android Studio and run the `app` configuration on your device or emulator.
