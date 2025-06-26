root := justfile_directory()

app-root := root / 'apps' / 'web-dioxus'

paper-root := root / 'paper'
paper-src := paper-root / 'main.typ'
paper-out := root / 'out' / 'typic-ocr-for-formula-generation.pdf'

export TYPST_ROOT := paper-root

[private]
default:
        @just --list --unsorted

[doc('Run all formatters')]
fmt: app-format paper-format
        nix fmt

[doc('Run all linters')]
lint: app-lint clippy

[group('rust')]
clippy: 
        cargo clippy --all --all-targets --all-features
        
[group('rust')]
[no-exit-message]
cargo-build package *args:
        cargo build -p {{ package }} {{ args }}
        
[group('app')]
app-release:
        cd {{ app-root }}; dx build --release

[group('app')]
app-watch:
        cd {{ app-root }}; dx serve --hot-reload --profile=dev

[group('app')]
app-format:
       cd {{ app-root }}; dx fmt

[group('app')]
app-lint:
        cd {{ app-root }}; cargo clippy --target wasm32-unknown-unknown

[group('training')]
train:
        poetry run train

[group('training')]
train_val_split:
        poetry run train_val_split

[group('training')]
symbols2svg:
        poetry run symbols2svg

[group('paper')]
paper-preview:
        typst-preview --root {{ paper-root }} {{ paper-src }}

[group('paper')]
paper-build:
        typst compile {{ paper-src }} {{ paper-out }}

[group('paper')]
paper-watch:
        typst watch {{ paper-src }} {{ paper-out }}

[group('paper')]
paper-format:
        typstyle -i format-all
