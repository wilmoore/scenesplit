//! SceneSplit: Extract semantically distinct still images from video.

mod config;
mod embeddings;
mod error;
mod model;
mod output;
mod processor;
mod segmentation;
mod video;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use config::{DetailLevel, QualityPreset};
use error::Error;
use model::ensure_model;
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
    long_about = "SceneSplit analyzes a video file and extracts representative frames that\ncapture meaningful visual changes. Output is written to a directory\ncontaining numbered images and a metadata.json file.\n\nOn first run, the embedding model (~100MB) is downloaded and cached."
)]
struct Args {
    /// Path to the input video file
    #[arg(value_name = "VIDEO")]
    input_video: PathBuf,

    /// Path to a custom ONNX model file (default: auto-download ResNet50)
    #[arg(long, short = 'm', value_name = "MODEL")]
    model: Option<PathBuf>,

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

    // Get model path (user-provided or auto-download)
    let model_path = match args.model {
        Some(path) => {
            if !path.exists() {
                return Err(Error::ModelLoad(format!(
                    "Model file not found: {}",
                    path.display()
                )));
            }
            path
        }
        None => ensure_model(args.quiet)?,
    };

    if !args.quiet {
        println!("SceneSplit v{}", VERSION);
        println!("Input: {}", args.input_video.display());
        println!("Model: {}", model_path.display());
        println!("Detail: {:?}", args.detail);
        println!("Quality: {:?}", args.quality);
        println!();
    }

    let processor = SceneSplitProcessor::new(
        args.detail,
        args.quality,
        args.output,
        model_path,
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
