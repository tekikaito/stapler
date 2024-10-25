# stapler

`stapler` is a command-line utility for merging multiple PDF files into a single PDF document. It offers options to specify input files, an output file, and optional compression for the output.

## Features

- Merge multiple PDF files into one.
- Optional compression for the output file.
- Command-line interface for easy integration into scripts or automation workflows.

## Installation

To use `stapler`, you need to have Rust installed. If Rust is not installed, [install it from here](https://www.rust-lang.org/tools/install).

Clone this repository and build the project:

```bash
git clone git@github.com:tekikaito/stapler.git
cd stapler
cargo build --release
```

The compiled binary will be located in the `target/release` directory.

## Usage

```bash
stapler --input <PDF_FILES> --output <OUTPUT_FILE> [--compress]
```

### Arguments

- `--input`, `-i` (required): List of input PDF files to merge.
- `--output`, `-o` (required): Name of the output PDF file.
- `--compress`, `-c` (optional): Enables compression for the output PDF file.

### Examples

Merge two PDF files without compression:

```bash
stapler --input file1.pdf file2.pdf --output merged.pdf
```

Merge and compress the output:

```bash
stapler --input file1.pdf file2.pdf --output merged.pdf --compress
```

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
