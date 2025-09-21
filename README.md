# Targets 🎯

`targets` is a simple tool that takes a set of input `values` at runtime and uses them to
populate one or more user-defined [mustache templates](https://mustache.github.io/mustache.5.html).

The motivating use-case for `targets` is simple and easy-to-use color scheme switching for all of your apps,
but it can be used anywhere that you want to populate one or more templates with
values that are decided at runtime.


## Usage

First, specify one or more targets in a configuration `toml` file:

```toml
[targets.ghostty]
template = "$XDG_CONFIG_HOME/targets/templates/ghostty.mustache"
out = "$XDG_CONFIG_HOME/ghostty/themes/base24"
hook = [
  "osascript",
  "-se",
  "$XDG_CONFIG_HOME/targets/hooks/reload_ghostty.applescript",
]

[targets.nvim]
template = "$XDG_CONFIG_HOME/targets/templates/vim.mustache"
out = "$XDG_CONFIG_HOME/nvim/colors/base24.vim"
hook = [
  "nvim",
  "--server",
  "~/.cache/nvim/server.pipe",
  "--remote-send",
  ":colorscheme base24<CR>",
]
```



Each target should have the following attributes: 
- `template`: The mustache template to use.
- `out`: The path to write populated template to. `targets` will create missing parent directories automatically.
- `hook` (optional): A shell command to run after writing the populated template.

By default, `targets` will read the configuration from `$XDG_CONFIG_HOME/targets/config.toml`,
but you can also specify a configuration file at runtime by passing `--config [PATH]`.

Next, specify some values in `json`, `yaml`, or `toml` format:

`values.json`:
```json
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

### Full list of CLI options
```
Usage: targets [OPTIONS]

Options:
  -f, --file <FILE>      Read values to populate `template` from a file insted of STDIN
      --format <FORMAT>  Format of the values read from stdin or from `file` [default: json] [possible values: json, yaml, toml]
  -c, --config <FILE>    Path to the configuration file that should be used (defaults to "$XDG_CONFIG_HOME/targets/config.toml")
  -h, --help             Print help
  -V, --version          Print version
```

### Using base16 / base24 / tinted themes & templates
[Chris Kempson](https://github.com/chriskempson/) and the maintainers of [Tinted Theming](https://github.com/tinted-theming/home)
(together with many other individual contributors) have curated a collection of [color schemes](https://github.com/tinted-theming/schemes)
and [templates](https://github.com/tinted-theming/home?tab=readme-ov-file#official-templates) for many different applications,
based on the [base16](https://github.com/chriskempson/base16) and [base24](https://github.com/tinted-theming/base24/blob/main/styling.md) 
specs. `targets` can use Tinted Theming templates directly: just download them and reference
them as a `template` in your configuration. The Tinted Theming color schemes must be modified to work with 
`targets`. This repository provides a script in [`extras/convert-tinted-scheme.nu`](./extra/convert-tinted-scheme.nu) 
to perform this conversion. If a base16 color scheme is provided, the script will use heuristics to generate
the missing colors to make it a base24 color scheme, so you can use any color
scheme with base24 templates.

To use the script, first install its dependencies: [`nushell`](https://www.nushell.sh/book/installation.html) and [`pastel`](https://github.com/sharkdp/pastel?tab=readme-ov-file#installation).
If you are using nix, the script is available as a flake output (see nix instructions below).
Then pipe a base16 or base24 yaml theme into the script:

```sh
cat rose-pine.yaml | convert-tinted-scheme.nu
```

or

```sh
curl -s https://raw.githubusercontent.com/tinted-theming/schemes/refs/heads/spec-0.11/base16/rose-pine.yaml | convert-tinted-scheme.nu
```

This means you can try out a new base16 or base24 color scheme to your system without changing your configuration:

```sh
curl -s https://raw.githubusercontent.com/tinted-theming/schemes/refs/heads/spec-0.11/base16/rose-pine.yaml | convert-tinted-scheme.nu | targets
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

Install via nix flakes:
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
        convert-tinted-scheme = inputs.targets.packages.${system}.convert-tinted-scheme;
      in
        with pkgs; {
          devShells.${system}.default = mkShell {packages = [ targets convert-tinted-scheme ];};
        }
    );
};
```
