<div align="center">

# Fuzzy Test Finder

</div>

> ⚠️ This project is still pre-alpha and not released yet.
> ⚠️ This project was developed on MAC and only tested on MAC.


`Fuzzy Test Finder` is a developer tool that helps you quickly find relevant tests for a given piece of code using fuzzy matching techniques. It is designed to improve developer productivity by reducing the time spent searching for tests.

Here's an example of using `fuzzy-test-finder`:
![demo.gif](./demo.gif)

## Quickstart

### Installation

First install the following dependencies:

```bash
brew install fzf
brew install expect
brew install bat
brew install ripgrep
```

Then build the executable:

```bash
cargo build --release
```

Create an alias for easier access:

```sh
# Example in .zshrc
alias fzt="<PATH>/FuzzyTestFinder/target/release/FzT"
```

Create a `.fzt` folder in `~/`

```bash
mkdir ~/.fzt
```

If you want to parse java gradle teste do the following steps:

```bash
cd parsers/java
./gradlew shadowJar
cp app/build/libs/app-all.jar ~/.fzt/fzt-java-parser.jar
```

### Usage

First you need to set the default language for the project. That way you only have to tell the tool once.

```bash
fzt --default rust 
```

Afterwards you can fuzzy find the tests. You can do that on multiple modes:

```bash
# Fuzzy find each test, items are <FILE_PATH>::<TEST_NAME>
# THis is the standart mode
fzt -m test
# or
fzt

# Fuzzy find each test in its default runtime name.
# So in case of cargo: cache::manager::tests::get_non_existing_entry
fzt -m runtime

# Fuzzy find files. It will run all the tests in the selected files.
fzt -m file

# Fuzzy find directories. It will run all the tests in the selected directories.
fzt -m directory

# Append mode that allows you selecting multiple times form different modes.
fzt -m append

# Select window for modes
fzt -m s

# Select a preview mode
fzt -p test
fzt -p directory
fzt -p file
# Select preview mode
fzt -p s

# Runs all tests
fzt --all

# Select from history (each mode has a dedicated history)
fzt -h
fzt -m directory -h
fzt -m append -h

# Run last item
fzt -l
fzt -m directory -l
fzt -m append -l

# Select from failed tests from last run (all failed test are saved in a set)
# You can then also select them in a preferred mode
# If -f is set the failed test stay unchanged
# They only get refreshed if you run fzt without a debugger option and -f
fzt -f
fzt -m directory -f
fzt -m append -f

# Run with debugger (currently only python)
# Set breakpoints with `breakpoint()` in the files
fzt -d pdb
# Get debugger selection window
fzt -d s

# clear cache
fzt --clear-cache

# Clear history
fzt --clear-history

# Run in verbose mode
fzt -v

# Parse arguments to runtime
fzt --all -- --locked ...

# Watch mode
# Run fzt in watch mode. It will re-run the last command when a file changes.
fzt --w
fzt --w -m directory
fzt -w --all


# See all test related to changed files
# Will pick up tests covering changed file since last run with -c or --covered
fzt -c
fzt --covered
```

## Supported languages

Currently supported languages:

- Python (pytest)
- Rust
- Java (JUnit with Gradle)

## License

This project is licensed under euther of

* Apache License, Version 2.0
* MIT license
