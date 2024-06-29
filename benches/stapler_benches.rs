use std::{ env::var, fs };

use criterion::{ black_box, criterion_group, criterion_main, BatchSize, Criterion };
use merge::{
    loader::fs::{ FileSystemMergingDestination, FileSystemMergingSource },
    FileSystemOptions,
    tests::create_sample_pdf,
};
use stapler::{ merge::{ self, tests::COMPRESS_OUTPUT_WHEN_TESTING }, stapler };

fn stapler_benchmark(c: &mut Criterion) {
    // get maximum from env variable or default to 100
    let max_files = var("CRITERION_MAX_FILES").unwrap_or("100".to_string()).parse::<u32>().unwrap();
    let testfiles_dir = format!("testfiles/from-bencharks/{}-inputs", max_files);

    let input_files: Vec<String> = (1..=max_files)
        .map(|index| format!("{}/input{}.pdf", testfiles_dir, index))
        .collect();
    let output_file = format!("{}/output.pdf", testfiles_dir);

    // ensure the testfiles directory exists
    fs::create_dir_all(testfiles_dir.clone()).unwrap();

    let file_options = FileSystemOptions {
        input_sources: input_files
            .iter()
            .map(|input_file| { FileSystemMergingSource { input_file } })
            .collect(),
        destination: FileSystemMergingDestination {
            output_file: &output_file,
        },
        compress: COMPRESS_OUTPUT_WHEN_TESTING,
    };

    c.bench_function(&format!("stapler running on {} files", max_files), |b| {
        b.iter_batched(
            || {
                file_options.input_sources
                    .iter()
                    .enumerate()
                    .for_each(|(index, source)| {
                        create_sample_pdf(&(index + 1).to_string())
                            .save(source.input_file)
                            .unwrap();
                    });
                black_box(file_options.clone())
            },
            |file_options| stapler(file_options),
            BatchSize::LargeInput
        )
    });

    let keep_output = var("CRITERION_KEEP_OUTPUT");
    if keep_output.is_err() || keep_output.unwrap() == "false" {
        fs::remove_dir_all(testfiles_dir).unwrap();
    }
}

criterion_group!(benches, stapler_benchmark);
criterion_main!(benches);
