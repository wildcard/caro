#!/usr/bin/env python3
"""
Analyze Kyaro's vocal characteristics.

Extracts acoustic features from dog vocalizations for timbre transfer.
"""

import argparse
import json
from pathlib import Path

import librosa
import numpy as np
import parselmouth
from parselmouth.praat import call
from tqdm import tqdm


def extract_pitch_features(sound: parselmouth.Sound) -> dict:
    """Extract pitch (F0) features using Praat."""
    pitch = sound.to_pitch()
    f0_values = pitch.selected_array["frequency"]
    f0_values = f0_values[f0_values > 0]  # Remove unvoiced frames

    if len(f0_values) == 0:
        return {"f0_mean": 0, "f0_std": 0, "f0_min": 0, "f0_max": 0, "f0_range": 0}

    return {
        "f0_mean": float(np.mean(f0_values)),
        "f0_std": float(np.std(f0_values)),
        "f0_min": float(np.min(f0_values)),
        "f0_max": float(np.max(f0_values)),
        "f0_range": float(np.max(f0_values) - np.min(f0_values)),
    }


def extract_formant_features(sound: parselmouth.Sound) -> dict:
    """Extract formant features (F1-F4)."""
    formant = sound.to_formant_burg(max_number_of_formants=4)

    features = {}
    for i in range(1, 5):
        values = []
        for t in formant.ts():
            try:
                val = formant.get_value_at_time(i, t)
                if not np.isnan(val):
                    values.append(val)
            except Exception:
                pass

        if values:
            features[f"f{i}_mean"] = float(np.mean(values))
            features[f"f{i}_std"] = float(np.std(values))
        else:
            features[f"f{i}_mean"] = 0
            features[f"f{i}_std"] = 0

    return features


def extract_harmonicity(sound: parselmouth.Sound) -> dict:
    """Extract harmonics-to-noise ratio."""
    harmonicity = sound.to_harmonicity()
    hnr_values = harmonicity.values.flatten()
    hnr_values = hnr_values[~np.isnan(hnr_values)]
    hnr_values = hnr_values[hnr_values > -200]  # Filter extreme values

    if len(hnr_values) == 0:
        return {"hnr_mean": 0, "hnr_std": 0}

    return {
        "hnr_mean": float(np.mean(hnr_values)),
        "hnr_std": float(np.std(hnr_values)),
    }


def extract_spectral_features(y: np.ndarray, sr: int) -> dict:
    """Extract spectral features using librosa."""
    # Spectral centroid (brightness)
    centroid = librosa.feature.spectral_centroid(y=y, sr=sr)[0]

    # Spectral bandwidth
    bandwidth = librosa.feature.spectral_bandwidth(y=y, sr=sr)[0]

    # Spectral rolloff
    rolloff = librosa.feature.spectral_rolloff(y=y, sr=sr)[0]

    # Spectral flatness
    flatness = librosa.feature.spectral_flatness(y=y)[0]

    # Zero crossing rate
    zcr = librosa.feature.zero_crossing_rate(y)[0]

    return {
        "spectral_centroid_mean": float(np.mean(centroid)),
        "spectral_centroid_std": float(np.std(centroid)),
        "spectral_bandwidth_mean": float(np.mean(bandwidth)),
        "spectral_rolloff_mean": float(np.mean(rolloff)),
        "spectral_flatness_mean": float(np.mean(flatness)),
        "zero_crossing_rate_mean": float(np.mean(zcr)),
    }


def extract_mfcc_features(y: np.ndarray, sr: int, n_mfcc: int = 13) -> dict:
    """Extract MFCC features."""
    mfccs = librosa.feature.mfcc(y=y, sr=sr, n_mfcc=n_mfcc)

    features = {}
    for i in range(n_mfcc):
        features[f"mfcc_{i}_mean"] = float(np.mean(mfccs[i]))
        features[f"mfcc_{i}_std"] = float(np.std(mfccs[i]))

    return features


def analyze_audio_file(audio_path: Path) -> dict:
    """
    Extract all acoustic features from an audio file.

    Args:
        audio_path: Path to audio file

    Returns:
        Dictionary of extracted features
    """
    # Load with librosa
    y, sr = librosa.load(audio_path, sr=44100, mono=True)

    # Load with Parselmouth
    sound = parselmouth.Sound(str(audio_path))

    # Extract all features
    features = {
        "file": audio_path.name,
        "duration": float(len(y) / sr),
        "sample_rate": sr,
    }

    # Pitch features
    features.update(extract_pitch_features(sound))

    # Formant features
    features.update(extract_formant_features(sound))

    # Harmonicity
    features.update(extract_harmonicity(sound))

    # Spectral features
    features.update(extract_spectral_features(y, sr))

    # MFCC features
    features.update(extract_mfcc_features(y, sr))

    return features


def compute_aggregate_features(all_features: list) -> dict:
    """Compute aggregate statistics across all files."""
    if not all_features:
        return {}

    # Get all numeric keys
    sample = all_features[0]
    numeric_keys = [k for k, v in sample.items() if isinstance(v, (int, float)) and k not in ("duration", "sample_rate")]

    aggregate = {}
    for key in numeric_keys:
        values = [f[key] for f in all_features if key in f and f[key] != 0]
        if values:
            aggregate[f"{key}_overall_mean"] = float(np.mean(values))
            aggregate[f"{key}_overall_std"] = float(np.std(values))

    # Total duration
    aggregate["total_duration"] = sum(f["duration"] for f in all_features)

    return aggregate


def analyze_directory(
    input_dir: str,
    output_file: str = "kyaro_features.json",
) -> None:
    """
    Analyze all audio files in a directory.

    Args:
        input_dir: Directory containing processed audio
        output_file: Output JSON file for features
    """
    input_path = Path(input_dir)

    # Find audio files
    audio_files = list(input_path.glob("*.wav"))

    if not audio_files:
        print(f"No WAV files found in {input_dir}")
        return

    print(f"Analyzing {len(audio_files)} audio files...")
    print()

    all_features = []

    for audio_file in tqdm(audio_files, desc="Extracting features"):
        try:
            features = analyze_audio_file(audio_file)
            all_features.append(features)
        except Exception as e:
            print(f"\nError analyzing {audio_file.name}: {e}")
            continue

    # Compute aggregate features
    aggregate = compute_aggregate_features(all_features)

    # Save results
    results = {
        "aggregate": aggregate,
        "files": all_features,
    }

    output_path = Path(output_file)
    with open(output_path, "w") as f:
        json.dump(results, f, indent=2)

    # Print summary
    print()
    print("=" * 60)
    print("Kyaro Vocal Analysis Summary")
    print("=" * 60)
    print(f"Files analyzed: {len(all_features)}")
    print(f"Total duration: {aggregate.get('total_duration', 0):.1f}s")
    print()
    print("Key Characteristics:")
    print(f"  Pitch (F0) mean:     {aggregate.get('f0_mean_overall_mean', 0):.1f} Hz")
    print(f"  Pitch range:         {aggregate.get('f0_range_overall_mean', 0):.1f} Hz")
    print(f"  Formant 1 (F1):      {aggregate.get('f1_mean_overall_mean', 0):.1f} Hz")
    print(f"  Formant 2 (F2):      {aggregate.get('f2_mean_overall_mean', 0):.1f} Hz")
    print(f"  Spectral centroid:   {aggregate.get('spectral_centroid_mean_overall_mean', 0):.1f} Hz")
    print(f"  Harmonicity (HNR):   {aggregate.get('hnr_mean_overall_mean', 0):.1f} dB")
    print()
    print(f"Results saved to: {output_path}")


def main():
    parser = argparse.ArgumentParser(
        description="Analyze Kyaro's vocal characteristics"
    )
    parser.add_argument(
        "--input",
        "-i",
        default="audio/processed",
        help="Directory with processed audio",
    )
    parser.add_argument(
        "--output",
        "-o",
        default="kyaro_features.json",
        help="Output JSON file",
    )

    args = parser.parse_args()

    analyze_directory(args.input, args.output)


if __name__ == "__main__":
    main()
