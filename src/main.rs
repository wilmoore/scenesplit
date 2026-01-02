//! SceneSplit: Extract semantically distinct still images from video.

mod config;
mod embeddings;
mod error;
mod output;
mod processor;
mod segmentation;
mod video;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use config::{DetailLevel, QualityPreset};
use error::Error;
use processor::SceneSplitProcessor;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Extract semantically distinct still images from a video.
///
/// SceneSplit analyzes a video file and extracts representative frames that
/// capture meaningful visual changes. Output is written to a directory
/// containing numbered images and a metadata.json file.
#[derive(Parser, Debug)]
#[command(name = "scenesplit")]
#[command(version = VERSION)]
#[command(about = "Extract semantically distinct still images from video")]
#[command(
    long_about = "SceneSplit analyzes a video file and extracts representative frames that\ncapture meaningful visual changes. Output is written to a directory\ncontaining numbered images and a metadata.json file."
)]
struct Args {
    /// Path to the input video file
    #[arg(value_name = "VIDEO")]
    input_video: PathBuf,

    /// Path to the ONNX model file (ResNet50 or similar feature extractor)
    #[arg(long, short = 'm', value_name = "MODEL")]
    model: PathBuf,

    /// Granularity level: 'key' (minimal), 'summary' (moderate), 'all' (comprehensive)
    #[arg(long, short = 'd', default_value = "summary", value_enum)]
    detail: DetailLevel,

    /// Processing quality: 'fast', 'balanced', or 'best'
    #[arg(long, short = 'q', default_value = "balanced", value_enum)]
    quality: QualityPreset,

    /// Output directory (default: ./scenesplit_output/)
    #[arg(long, short = 'o', value_name = "DIR")]
    output: Option<PathBuf>,

    /// Suppress progress output
    #[arg(long, short = 's')]
    quiet: bool,
}

fn progress_callback(stage: &str, current: usize, total: usize) {
    if total > 0 {
        println!("{}... ({}/{})", stage, current, total);
    } else {
        println!("{}...", stage);
    }
}

fn run(args: Args) -> Result<(), Error> {
    // Validate input file exists
    if !args.input_video.exists() {
        return Err(Error::VideoNotFound(args.input_video));
    }

    // Validate model file exists
    if !args.model.exists() {
        return Err(Error::ModelLoad(format!(
            "Model file not found: {}",
            args.model.display()
        )));
    }

    if !args.quiet {
        println!("SceneSplit v{}", VERSION);
        println!("Input: {}", args.input_video.display());
        println!("Model: {}", args.model.display());
        println!("Detail: {:?}", args.detail);
        println!("Quality: {:?}", args.quality);
        println!();
    }

    let processor = SceneSplitProcessor::new(
        args.detail,
        args.quality,
        args.output,
        args.model,
    );

    let callback = if args.quiet {
        None
    } else {
        Some(progress_callback)
    };

    let result = processor.process(&args.input_video, callback)?;

    if !args.quiet {
        println!();
        println!("{}", "=".repeat(50));
        println!("Extracted {} stills", result.frames_extracted);
        println!("Output written to {}/", result.output_dir.display());
        println!("{}", "=".repeat(50));
    }

    Ok(())
}

fn main() -> ExitCode {
    let args = Args::parse();

    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}
