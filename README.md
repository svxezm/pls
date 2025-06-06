# pls

A simple remake of the classic `ls` program, written in Rust.

## Installing

1. Clone this repository: https://github.com/svxezm/pls.git
2. Make sure you have Rust and Cargo installed. If not, have a look [here](https://rustup.rs)
3. In the root project directory, run `cargo build --release`
4. Declare an alias to `target/release/pls` for your shell

### An alias example for Zsh

`alias -- 'pls'='cd <project_path>/pls && ./target/release/pls'`

## Usage

You can use a few arguments to use this program

- `<target>` - specifies the target directory
- `-r` - enable recursive size calculation for directories
