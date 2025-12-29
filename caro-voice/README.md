# Caro Voice Synthesis

Voice synthesis sub-project for Caro CLI - creating a unique human voice seeded from Kyaro's (the real Shiba Inu) vocalizations.

## Concept

Caro is the CLI assistant, inspired by Kyaro the Shiba Inu. This project creates Caro's voice by:

1. Recording Kyaro's real vocalizations (barks, whines, happy sounds)
2. Extracting timbral characteristics using audio analysis
3. Transferring those characteristics to a human voice using neural timbre transfer
4. Integrating the voice into the Caro CLI

The result: a friendly, unique voice that carries subtle "Kyaro DNA" in its timbre.

## Quick Start

```bash
# Setup environment
cd caro-voice
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Record Kyaro audio (see PLAN.md for details)
# Place recordings in audio/raw/

# Preprocess audio
python scripts/preprocess.py

# Analyze features
python scripts/analyze.py

# Train timbre model
python scripts/train_ddsp.py

# Generate voice
python scripts/generate.py "Hello, I'm Caro"
```

## Documentation

- **[RESEARCH.md](RESEARCH.md)** - Deep research on voice synthesis approaches
- **[PLAN.md](PLAN.md)** - Step-by-step implementation plan

## Architecture

```
[Kyaro Audio] → [DDSP Timbre Model] → [Kyaro Timbre Vector]
                                              ↓
[Text Input] → [F5-TTS] → [Base Audio] → [Timbre Transfer] → [Caro Voice]
```

## Key Technologies

| Component | Technology | Purpose |
|-----------|------------|---------|
| Audio Analysis | librosa, parselmouth | Extract Kyaro's vocal features |
| Timbre Transfer | DDSP (Google Magenta) | Apply dog timbre to human voice |
| Base TTS | F5-TTS | High-quality text-to-speech |
| CLI Integration | rodio (Rust) | Audio playback in CLI |

## Directory Structure

```
caro-voice/
├── audio/
│   ├── raw/           # Kyaro recordings
│   ├── processed/     # Cleaned audio
│   └── generated/     # Caro voice clips
├── models/            # Trained models
├── scripts/           # Python scripts
├── notebooks/         # Jupyter analysis
└── requirements.txt
```

## Status

Phase 1: Audio Collection (not started)

## License

MIT License - Part of the Caro project
