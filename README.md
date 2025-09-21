# Targets 🎯

`targets` is a simple tool that takes a set of input `values` at runtime and uses them to
populate one or more user-defined [mustache templates](https://mustache.github.io/mustache.5.html).

The motivating use-case for `targets` is simple and easy-to-use color scheme switching for all of your apps,
but it can be used anywhere that you want to populate one or more templates with
values that are decided at runtime.


## Usage

First, specify one or more targets in a configuration `toml` file:

```toml
# config.toml
[targets.ghostty]
template = "$XDG_CONFIG_HOME/targets/templates/ghostty.mustache"
out = "$XDG_CONFIG_HOME/ghostty/themes/base24"
hook = [
  "osascript",
  "-se",
  "$XDG_CONFIG_HOME/targets/hooks/reload_ghostty.applescript",
]
```

Each target should have the following attributes: 
- `template`: The mustache template to use.
- `out`: The path to write populated template to.
- `hook` (optional): A shell command to run after writing the populated template.

By default, `targets` will read the configuration from `$XDG_CONFIG_HOME/targets/config.toml`,
but you can also specify a configuration file at runtime by passing `--config [PATH]`.

Next, specify some values in `json`, `yaml`, or `toml` format:
```jsonc
# values.json
{
  "scheme-system": "base16",
  "scheme-name": "Rosé Pine",
  "scheme-author": "Emilia Dunfelt <edun@dunfelt.se>",
  "scheme-slug": "rose-pine",
  "scheme-variant": "dark",
  "base00-hex": "191724",
  "base01-hex": "1f1d2e",
  "base02-hex": "26233a",
  "base03-hex": "6e6a86",
  "base04-hex": "908caa",
  "base05-hex": "e0def4",
  "base06-hex": "e0def4",
  "base07-hex": "524f67",
  "base08-hex": "eb6f92",
  "base09-hex": "f6c177",
  "base0A-hex": "ebbcba",
  "base0B-hex": "31748f",
  "base0C-hex": "9ccfd8",
  "base0D-hex": "c4a7e7",
  "base0E-hex": "f6c177",
  "base0F-hex": "524f67",
  "base10-hex": "000000",
  "base11-hex": "000000",
  "base12-hex": "f59cb1",
  "base13-hex": "fcd49f",
  "base14-hex": "f2d0ce",
  "base15-hex": "749caf",
  "base16-hex": "badde4",
  "base17-hex": "d6c1ee"
}
```

Finally, run `targets` and pass your values in:
```sh
targets --file values.json
```

Values can also be piped from `stdin`:
```sh
cat values.json | targets --format json
```

### Full list of options
```
Usage: targets [OPTIONS]

Options:
  -f, --file <FILE>      Read values to populate `template` from a file insted of STDIN
      --format <FORMAT>  Format of the values read from stdin or from `file` [default: json] [possible values: json, yaml, toml]
  -c, --config <FILE>    Path to the configuration file that should be used (defaults to "$XDG_CONFIG_HOME/targets/config.toml")
  -h, --help             Print help
  -V, --version          Print version
```


## Installation

### Cargo
1. Make sure you have installed the [rust toolchain](https://www.rust-lang.org/tools/install)
2. Clone this repository and `cd` into it
3. Compile and install the binary: `cargo install --path .`


### Nix
To try out temporarily:
```sh
nix shell 'github:LoganWalls/targets'
```

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    targets = {
        url = "github:LoganWalls/targets";
        inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {nixpkgs, ...}@inputs: let
    inherit (nixpkgs) lib;
    withSystem = f:
      lib.fold lib.recursiveUpdate {}
      (map f ["x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin"]);
  in
    withSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        targets = inputs.targets.packages.${system}.default;
        from-tinted-scheme = inputs.targets.packages.${system}.from-tinted-scheme;
      in
        with pkgs; {
          devShells.${system}.default = mkShell {packages = [ targets from-tinted-scheme ];};
        }
    );
};
```
