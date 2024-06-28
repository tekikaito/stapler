use std::fs;

use criterion::{ black_box, criterion_group, criterion_main, BatchSize, Criterion };
use merge::{
    loader::fs::{ FileSystemMergingDestination, FileSystemMergingSource },
    FileSystemOptions,
    tests::create_sample_pdf,
};
use stapler::{ merge, stapler };

fn stapler_benchmark(c: &mut Criterion) {
    let testfiles_dir = "testfiles";
    let input_files: Vec<String> = (1..=100)
        .map(|index| format!("{}/input{}.pdf", testfiles_dir, index))
        .collect();
    let output_file = format!("{}/output.pdf", testfiles_dir);

    // ensure the testfiles directory exists
    fs::create_dir_all(testfiles_dir).unwrap();

    let file_options = FileSystemOptions {
        input_sources: input_files
            .iter()
            .enumerate()
            .map(|(index, input_file)| {
                create_sample_pdf(&(index + 1).to_string())
                    .save(input_file)
                    .unwrap();
                FileSystemMergingSource {
                    input_file,
                }
            })
            .collect(),
        destination: FileSystemMergingDestination {
            output_file: &output_file,
        },
    };

    c.bench_function("stapler", |b| {
        b.iter_batched(
            || black_box(file_options.clone()),
            |file_options| {
                // Code to be benchmarked goes here
                stapler(file_options)
            },
            BatchSize::LargeInput
        )
    });

    let keep_output = std::env::var("CRITERION_KEEP_OUTPUT");
    if keep_output.is_err() || keep_output.unwrap() == "false" {
        fs::remove_dir_all(testfiles_dir).unwrap();
    }
}

criterion_group!(benches, stapler_benchmark);
criterion_main!(benches);
