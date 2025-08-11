# stapler

`stapler` is a command-line utility for merging multiple PDF files into a single PDF document. It offers options to specify input files, an output file, and optional compression for the output.

## Features

- Merge multiple PDF files into one.
- **Glob pattern support**: Use wildcard patterns like `*.pdf` to match multiple files.
- **Cross-platform**: Works on Windows, macOS, and Linux with consistent glob behavior.
- **Hidden file support**: Include dotfiles using patterns like `.*.pdf`.
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

- `--input`, `-i` (required): List of input PDF files to merge. Supports glob patterns (e.g., `*.pdf`, `/path/to/*.pdf`).
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

**Using glob patterns:**

Merge all PDF files in a directory:

```bash
stapler --input "*.pdf" --output merged.pdf
```

Merge all PDF files in a specific directory:

```bash
stapler --input "/path/to/documents/*.pdf" --output merged.pdf
```

Include hidden PDF files:

```bash
stapler --input "*.pdf" ".*.pdf" --output merged.pdf
```

Mix glob patterns with literal file paths:

```bash
stapler --input "reports/*.pdf" "summary.pdf" --output final_report.pdf
```

**Note:** On Unix-like systems, wrap glob patterns in quotes to prevent shell expansion. On Windows, quotes are recommended but not always necessary.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
