#!/usr/bin/env python3
"""
Preprocess Kyaro audio recordings.

Normalizes, trims silence, and prepares audio for feature extraction and training.
"""

import argparse
from pathlib import Path

import librosa
import numpy as np
import soundfile as sf
from tqdm import tqdm


def preprocess_audio(
    input_path: Path,
    output_path: Path,
    target_sr: int = 44100,
    top_db: float = 30.0,
    normalize: bool = True,
) -> dict:
    """
    Preprocess a single audio file.

    Args:
        input_path: Path to input audio file
        output_path: Path to save processed audio
        target_sr: Target sample rate
        top_db: Threshold for silence trimming
        normalize: Whether to normalize audio

    Returns:
        dict with processing statistics
    """
    # Load audio
    y, sr = librosa.load(input_path, sr=target_sr, mono=True)

    original_duration = len(y) / sr

    # Trim silence from beginning and end
    y_trimmed, index = librosa.effects.trim(y, top_db=top_db)

    # Normalize amplitude
    if normalize:
        y_trimmed = librosa.util.normalize(y_trimmed)

    trimmed_duration = len(y_trimmed) / sr

    # Save processed audio
    sf.write(output_path, y_trimmed, target_sr)

    return {
        "original_duration": original_duration,
        "trimmed_duration": trimmed_duration,
        "removed_seconds": original_duration - trimmed_duration,
        "sample_rate": target_sr,
    }


def process_directory(
    input_dir: str,
    output_dir: str,
    target_sr: int = 44100,
    extensions: tuple = (".wav", ".mp3", ".flac", ".m4a"),
) -> None:
    """
    Process all audio files in a directory.

    Args:
        input_dir: Directory containing raw audio files
        output_dir: Directory for processed audio
        target_sr: Target sample rate
        extensions: Audio file extensions to process
    """
    input_path = Path(input_dir)
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    # Find all audio files
    audio_files = []
    for ext in extensions:
        audio_files.extend(input_path.glob(f"*{ext}"))
        audio_files.extend(input_path.glob(f"*{ext.upper()}"))

    if not audio_files:
        print(f"No audio files found in {input_dir}")
        print(f"Supported extensions: {extensions}")
        return

    print(f"Found {len(audio_files)} audio files")
    print(f"Target sample rate: {target_sr} Hz")
    print()

    total_original = 0
    total_trimmed = 0

    for audio_file in tqdm(audio_files, desc="Processing"):
        output_file = output_path / f"{audio_file.stem}.wav"

        try:
            stats = preprocess_audio(audio_file, output_file, target_sr)
            total_original += stats["original_duration"]
            total_trimmed += stats["trimmed_duration"]
        except Exception as e:
            print(f"\nError processing {audio_file.name}: {e}")
            continue

    print()
    print("=" * 50)
    print("Processing Complete")
    print("=" * 50)
    print(f"Total original duration: {total_original:.1f}s ({total_original/60:.1f} min)")
    print(f"Total trimmed duration:  {total_trimmed:.1f}s ({total_trimmed/60:.1f} min)")
    print(f"Silence removed:         {total_original - total_trimmed:.1f}s")
    print(f"Output directory:        {output_path}")


def main():
    parser = argparse.ArgumentParser(
        description="Preprocess Kyaro audio recordings for voice synthesis"
    )
    parser.add_argument(
        "--input",
        "-i",
        default="audio/raw",
        help="Input directory with raw audio files",
    )
    parser.add_argument(
        "--output",
        "-o",
        default="audio/processed",
        help="Output directory for processed audio",
    )
    parser.add_argument(
        "--sample-rate",
        "-sr",
        type=int,
        default=44100,
        help="Target sample rate (default: 44100)",
    )

    args = parser.parse_args()

    process_directory(args.input, args.output, args.sample_rate)


if __name__ == "__main__":
    main()
