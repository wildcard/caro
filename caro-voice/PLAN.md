# Caro Voice Implementation Plan

## Overview

Create "Caro's Voice" - a unique human TTS voice seeded from Kyaro (the real Shiba Inu).

**Timeline**: 4 phases, iterative development
**Primary Approach**: DDSP Timbre Transfer + F5-TTS

---

## Phase 1: Audio Collection & Setup

### 1.1 Record Kyaro Audio

**Equipment**:
- Smartphone with voice recorder app (or dedicated recorder)
- Quiet room, minimal background noise
- Position device 1-2 feet from Kyaro

**Recording Specifications**:
```
Format: WAV (uncompressed)
Sample Rate: 44100 Hz or 48000 Hz
Bit Depth: 16-bit or 24-bit
Channels: Mono
```

**Audio to Collect**:
| Category | Target Duration | Notes |
|----------|-----------------|-------|
| Standard barks | 2-3 min | Single barks, varied |
| Alert barks | 1-2 min | Sharp, attention-getting |
| Happy sounds | 2 min | Playful, excited |
| Whines/cries | 1-2 min | Emotional range |
| Grumbles | 1-2 min | Low, sustained |
| **Total** | **8-12 min** | |

**Tips**:
- Use treats/toys to elicit varied sounds
- Record multiple sessions if needed
- Label files clearly: `kyaro_bark_001.wav`, etc.

### 1.2 Environment Setup

```bash
# Create project structure
cd /home/user/caro/caro-voice
mkdir -p audio/raw audio/processed models/ddsp models/tts scripts notebooks

# Python environment
python3 -m venv venv
source venv/bin/activate

# Core dependencies
pip install torch torchaudio librosa soundfile
pip install parselmouth opensmile praat-parselmouth
pip install matplotlib numpy scipy pandas
pip install jupyter ipykernel

# DDSP
pip install ddsp

# F5-TTS (when ready)
pip install f5-tts
```

---

## Phase 2: Feature Extraction & Analysis

### 2.1 Audio Preprocessing

```python
# scripts/preprocess.py
import librosa
import soundfile as sf
from pathlib import Path

def preprocess_audio(input_dir, output_dir, sr=44100):
    """Normalize and clean Kyaro audio samples."""
    input_path = Path(input_dir)
    output_path = Path(output_dir)
    output_path.mkdir(exist_ok=True)

    for audio_file in input_path.glob("*.wav"):
        # Load audio
        y, _ = librosa.load(audio_file, sr=sr, mono=True)

        # Normalize
        y = librosa.util.normalize(y)

        # Trim silence
        y, _ = librosa.effects.trim(y, top_db=30)

        # Save
        output_file = output_path / audio_file.name
        sf.write(output_file, y, sr)
        print(f"Processed: {audio_file.name}")
```

### 2.2 Feature Analysis

```python
# scripts/analyze_features.py
import librosa
import parselmouth
import numpy as np

def extract_kyaro_features(audio_path):
    """Extract acoustic features from Kyaro's vocalizations."""

    # Load with librosa
    y, sr = librosa.load(audio_path, sr=44100)

    # Load with Parselmouth for formants
    sound = parselmouth.Sound(audio_path)

    features = {}

    # 1. Pitch (F0)
    pitch = sound.to_pitch()
    features['f0_mean'] = np.nanmean(pitch.selected_array['frequency'])
    features['f0_std'] = np.nanstd(pitch.selected_array['frequency'])

    # 2. Formants
    formant = sound.to_formant_burg()
    features['f1_mean'] = np.mean([formant.get_value_at_time(1, t)
                                    for t in formant.ts()])
    features['f2_mean'] = np.mean([formant.get_value_at_time(2, t)
                                    for t in formant.ts()])
    features['f3_mean'] = np.mean([formant.get_value_at_time(3, t)
                                    for t in formant.ts()])

    # 3. MFCCs
    mfccs = librosa.feature.mfcc(y=y, sr=sr, n_mfcc=13)
    features['mfcc_mean'] = np.mean(mfccs, axis=1)

    # 4. Spectral features
    features['spectral_centroid'] = np.mean(
        librosa.feature.spectral_centroid(y=y, sr=sr))
    features['spectral_bandwidth'] = np.mean(
        librosa.feature.spectral_bandwidth(y=y, sr=sr))

    # 5. Harmonicity
    harmonicity = sound.to_harmonicity()
    features['hnr_mean'] = np.nanmean(harmonicity.values)

    return features
```

### 2.3 Visualization & Understanding

```python
# notebooks/kyaro_analysis.ipynb
# Create visualizations of Kyaro's vocal characteristics
# - Spectrograms of different vocalization types
# - Pitch contours
# - Formant tracks
# - MFCC heatmaps
```

---

## Phase 3: Timbre Model Training

### 3.1 DDSP Timbre Transfer

```bash
# Clone DDSP
cd /home/user/caro/caro-voice
git clone https://github.com/magenta/ddsp.git ddsp-repo

# Use the timbre transfer notebook as starting point
cp ddsp-repo/ddsp/colab/demos/timbre_transfer.ipynb notebooks/
```

**Training Steps**:

1. **Prepare Dataset**:
   - Combine all processed Kyaro audio
   - Create train/validation split (90/10)

2. **Train Autoencoder**:
   ```python
   # The DDSP autoencoder learns Kyaro's timbre
   # Key hyperparameters:
   # - n_samples: length of audio segments
   # - sample_rate: 16000 (DDSP default) or 44100
   # - n_harmonic_distribution: 60
   # - n_noise_magnitudes: 65
   ```

3. **Evaluate Model**:
   - Generate resynthesized Kyaro audio
   - Compare spectrograms
   - Listen for quality

### 3.2 Alternative: StyleTTS2 Style Extraction

```python
# If DDSP doesn't work well, try StyleTTS2
# Extract style vectors from Kyaro audio

from styletts2 import compute_style

# Load Kyaro audio
kyaro_style = compute_style("audio/processed/kyaro_combined.wav")

# This style vector can be used for TTS generation
# Though it's designed for human speech, interesting results possible
```

---

## Phase 4: Voice Synthesis & Integration

### 4.1 Generate Caro Voice

**Option A: DDSP + Base TTS**
```
[Text] → [F5-TTS (neutral voice)] → [Base Audio] → [DDSP Kyaro Timbre] → [Caro Voice]
```

**Option B: Custom Fine-tuned Model**
```python
# Fine-tune F5-TTS on Kyaro-transferred audio
# This creates a model that directly generates Caro's voice
```

### 4.2 Generate Voice Clips for CLI

**Priority Phrases**:
```
# Informational
"Command generated"
"Executing command"
"Command completed"

# Safety
"Warning: potentially dangerous command"
"This command requires confirmation"
"Operation cancelled"

# Errors
"Command failed"
"Unable to generate command"
"Backend not available"

# Personality
"Hello, I'm Caro"
"How can I help?"
"Goodbye"
```

### 4.3 CLI Integration Options

**Option 1: Pre-generated Audio** (Simplest)
```rust
// Embed audio clips in binary
const AUDIO_GENERATED: &[u8] = include_bytes!("../audio/generated.wav");

// Play using rodio crate
fn play_notification(clip: &[u8]) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let source = Decoder::new(Cursor::new(clip)).unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();
}
```

**Option 2: Local TTS** (Flexible)
```rust
// Use ONNX runtime for inference
// F5-TTS has ONNX export capability
use ort::{Session, Value};

fn generate_speech(text: &str, model: &Session) -> Vec<f32> {
    // Run inference
    // Return audio samples
}
```

**Option 3: HTTP API** (Scalable)
```rust
// Call voice synthesis server
async fn speak(text: &str) -> Result<Vec<u8>> {
    let response = reqwest::Client::new()
        .post("http://localhost:8000/synthesize")
        .json(&json!({"text": text, "voice": "caro"}))
        .send()
        .await?;
    Ok(response.bytes().await?.to_vec())
}
```

---

## Directory Structure

```
caro-voice/
├── RESEARCH.md          # This research document
├── PLAN.md              # This implementation plan
├── README.md            # Quick start guide
├── audio/
│   ├── raw/             # Original Kyaro recordings
│   ├── processed/       # Cleaned & normalized audio
│   └── generated/       # Caro voice clips
├── models/
│   ├── ddsp/            # DDSP timbre model
│   ├── tts/             # TTS model (F5-TTS, etc.)
│   └── onnx/            # Exported ONNX models
├── scripts/
│   ├── preprocess.py    # Audio preprocessing
│   ├── analyze.py       # Feature extraction
│   ├── train_ddsp.py    # DDSP training
│   └── generate.py      # Voice generation
├── notebooks/
│   ├── kyaro_analysis.ipynb
│   ├── timbre_transfer.ipynb
│   └── voice_generation.ipynb
├── requirements.txt
└── venv/                # Python virtual environment
```

---

## Dependencies

### requirements.txt
```
# Core
torch>=2.0
torchaudio>=2.0
numpy>=1.24
scipy>=1.10

# Audio Processing
librosa>=0.10
soundfile>=0.12
pydub>=0.25
parselmouth>=0.4
praat-parselmouth>=0.4
opensmile>=2.4

# DDSP
ddsp>=3.0
tensorflow>=2.10  # DDSP dependency

# TTS
f5-tts>=1.0

# Visualization
matplotlib>=3.7
seaborn>=0.12
jupyter>=1.0
ipykernel>=6.0

# Utilities
tqdm>=4.65
pandas>=2.0
pyyaml>=6.0
```

---

## Milestones

| Milestone | Description | Deliverable |
|-----------|-------------|-------------|
| M1 | Kyaro audio collected | 8-12 min of recordings |
| M2 | Environment setup | Working Python environment |
| M3 | Feature analysis complete | Feature report + visualizations |
| M4 | DDSP model trained | Kyaro timbre model |
| M5 | Voice prototype | First Caro voice samples |
| M6 | Voice refinement | Production-quality voice |
| M7 | CLI integration | Voice in Caro CLI |

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Kyaro audio quality poor | High | Multiple recording sessions, professional mic |
| DDSP doesn't capture dog timbre | High | Try StyleTTS2, AudioLDM alternatives |
| Voice sounds unnatural | Medium | Reduce timbre transfer strength |
| Too dog-like, not human | Medium | Blend with stronger human base |
| Performance too slow | Medium | Pre-generate clips, use ONNX |
| Large model size | Low | Quantization, distillation |

---

## Success Metrics

1. **Kyaro DNA**: Blind listening test - can listeners identify "something unique" about the voice?
2. **Naturalness**: MOS score > 3.5 (scale 1-5)
3. **Intelligibility**: Word error rate < 5%
4. **Character**: Described as "friendly", "helpful", "unique"
5. **Performance**: < 500ms generation for short phrases

---

## References

- [DDSP GitHub](https://github.com/magenta/ddsp)
- [F5-TTS GitHub](https://github.com/SWivid/F5-TTS)
- [StyleTTS2 GitHub](https://github.com/yl4579/StyleTTS2)
- [Librosa Documentation](https://librosa.org/doc/latest/index.html)
- [Parselmouth (Praat in Python)](https://parselmouth.readthedocs.io/)
