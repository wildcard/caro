# Caro Voice Synthesis Research

## Project Goal

Create a unique human voice for Caro (the CLI assistant) that is **seeded from Kyaro's real dog vocalizations**. The voice should:
- Sound like a natural human voice for TTS output
- Carry subtle timbral characteristics extracted from Kyaro's barks/vocalizations
- Be distinctive and recognizable as "Caro's voice"
- Work for announcing command generation, confirmations, and warnings

---

## Research Summary

### The Challenge: Cross-Species Voice Synthesis

This is a novel challenge that sits at the intersection of:
1. **Bioacoustic feature extraction** - extracting meaningful characteristics from dog vocalizations
2. **Timbre transfer** - applying those characteristics to human speech
3. **Voice synthesis/TTS** - generating natural-sounding speech output

No existing tool does this directly. We need a **multi-stage pipeline**.

---

## Technical Approaches Evaluated

### Approach 1: Timbre Transfer Pipeline (Recommended)

**Concept**: Extract timbral features from Kyaro's vocalizations, then apply them as style/timbre modifications to a base human voice.

**Pipeline**:
```
[Kyaro Audio] → [Feature Extraction] → [Timbre Vector]
                                              ↓
[Text Input] → [Base TTS] → [Base Audio] → [Timbre Transfer] → [Caro Voice]
```

**Tools**:
- **DDSP (Google Magenta)**: Differentiable Digital Signal Processing for timbre transfer
  - Can train on <13 minutes of audio
  - Extracts F0 and loudness, resynthesizes with target timbre
  - Open source: https://github.com/magenta/ddsp

- **Qosmo Timbre Transfer**: Real-time timbre transfer using VAE + adversarial learning
  - Can convert any sound into target timbre
  - Research collaboration with Neutone (2024-2025)

**Pros**: Most flexible, can create unique voice characteristics
**Cons**: Requires experimentation to find right balance

---

### Approach 2: Style-Guided TTS with Animal Audio Reference

**Concept**: Use TTS models that accept reference audio for style, providing Kyaro audio as style reference.

**Tools**:

- **StyleTTS2**: Human-level TTS with style diffusion
  - Uses reference audio to extract style vectors
  - `compute_style()` function extracts embeddings from any audio
  - Could potentially extract "style" from dog vocalizations
  - GitHub: https://github.com/yl4579/StyleTTS2

- **OpenVoice**: Instant voice cloning with tone color extraction
  - Separates content from tone color
  - Tone color converter could theoretically accept non-speech audio
  - May produce interesting hybrid results

**Pros**: Simpler pipeline, fewer components
**Cons**: Models optimized for human speech; results may be unpredictable

---

### Approach 3: Feature Extraction → Voice Design

**Concept**: Analyze Kyaro's vocalizations for specific acoustic features, then use those parameters to design a synthetic voice.

**Feature Extraction Methods**:
- **MFCC**: Mel-frequency cepstral coefficients (pitch, timbre)
- **Formants**: First 3 formants capture vocal tract characteristics
- **Harmonicity**: Harmonics-to-noise ratio
- **Spectral features**: Energy distribution across frequency bands

**Voice Design Tools**:
- **ElevenLabs Voice Design**: Create voices with adjustable age, accent, settings
  - API available for programmatic voice design
  - Could map dog vocal features to voice parameters

- **Bark (Suno)**: Text-prompted audio with non-verbal sounds
  - Can generate laughter, sighs, music
  - Open source: https://github.com/suno-ai/bark
  - Could be prompted to include dog-like characteristics

**Pros**: Most control over final result
**Cons**: Indirect mapping, requires interpretation of dog features

---

### Approach 4: Hybrid Neural Vocoder

**Concept**: Train a custom neural vocoder on a mix of human speech + Kyaro audio to create a unique synthesis model.

**Tools**:
- **AudioLDM 2**: Text-to-audio with style transfer in zero-shot fashion
- **DiffWave**: Diffusion-based audio synthesis, excels at unconditional generation
- **HiFi-GAN/BigVGAN**: Neural vocoders that can be fine-tuned

**Research Reference**:
- "When Humans Growl and Birds Speak" (arXiv 2505.24336)
  - Human to Non-Human Voice Conversion (H2NH-VC)
  - CVAE framework with VITS architecture
  - 44.1kHz high-quality audio transformation
  - Handles dog barks, birdsong, growls

**Pros**: Could create truly unique voice
**Cons**: Most complex, requires significant training data and compute

---

## Recommended Implementation Strategy

### Phase 1: Audio Collection & Preparation

**Collect Kyaro Audio**:
- Record diverse vocalizations: barks, whines, happy sounds, alert sounds
- Aim for 5-15 minutes of varied samples
- Record at 44.1kHz or 48kHz, 16-bit minimum
- Clean, low-noise environment

**Audio Requirements**:
| Type | Duration | Purpose |
|------|----------|---------|
| Short barks | 2-3 min | Core timbre extraction |
| Sustained sounds | 1-2 min | Formant analysis |
| Emotional range | 2-3 min | Style variation |
| Playful sounds | 2-3 min | Personality traits |

### Phase 2: Feature Extraction & Analysis

**Tools**:
```python
# Required libraries
librosa          # Audio analysis
parselmouth      # Praat-like formant extraction
scipy            # Signal processing
opensmile        # Standard audio feature extraction
torch            # ML framework
```

**Extract Features**:
1. **Pitch contour (F0)**: Fundamental frequency patterns
2. **Formants (F1-F3)**: Vocal tract resonances
3. **MFCCs**: Timbral characteristics
4. **Spectral centroid**: Brightness/warmth
5. **Harmonicity**: Tonal quality

### Phase 3: Voice Prototype Development

**Option A - DDSP Pipeline** (Recommended First):
```bash
# Clone DDSP
git clone https://github.com/magenta/ddsp

# Train timbre model on Kyaro audio
# Use timbre_transfer.ipynb as starting point
```

**Option B - StyleTTS2 Experiment**:
```bash
# Clone StyleTTS2
git clone https://github.com/yl4579/StyleTTS2

# Extract style from Kyaro audio
# Generate speech with dog-derived style vectors
```

### Phase 4: Base Voice Selection

Select a human voice foundation:
- **F5-TTS**: Best open-source quality (335M params, zero-shot)
- **Fish Speech 1.5**: Fast inference (RTF 1:15 on RTX 4090)
- **Tortoise TTS**: High quality, slower generation

### Phase 5: Integration & Refinement

1. Apply timbre characteristics to base voice
2. Fine-tune balance (too much = unintelligible, too little = no personality)
3. Test with Caro's typical utterances:
   - "Generated command: ls -la"
   - "Warning: This command modifies system files"
   - "Command executed successfully"

### Phase 6: Rust Integration

Options for Caro CLI:
1. **Pre-generated audio clips**: Most efficient, limited flexibility
2. **Local TTS with custom model**: Maximum quality, larger binary
3. **HTTP API to voice service**: Flexible, requires network

---

## Tool Comparison Matrix

| Tool | Quality | Speed | Ease | Animal Audio Support | License |
|------|---------|-------|------|---------------------|---------|
| **DDSP** | High | Fast | Medium | Excellent (timbre transfer) | Apache 2.0 |
| **StyleTTS2** | Excellent | Medium | Medium | Experimental | MIT |
| **F5-TTS** | Excellent | Fast | Easy | Limited | MIT |
| **Bark** | Good | Slow | Easy | Non-verbal sounds | MIT |
| **Fish Speech** | Excellent | Very Fast | Easy | Limited | Apache 2.0 |
| **Tortoise** | Excellent | Very Slow | Medium | Limited | Apache 2.0 |
| **OpenVoice** | Good | Fast | Easy | Tone color only | MIT |
| **ElevenLabs** | Best | Fast | Easy | Via API | Commercial |

---

## Hardware Requirements

**Minimum**:
- 8GB VRAM GPU (RTX 3070 or better)
- 16GB RAM
- 50GB storage

**Recommended**:
- 24GB VRAM GPU (RTX 4090)
- 32GB RAM
- 100GB SSD

**Apple Silicon**:
- M1 Pro/Max or better
- 32GB unified memory
- MLX compatible for some models

---

## Key Resources

### Papers
- "DDSP: Differentiable Digital Signal Processing" (ICLR 2020)
- "StyleTTS 2: Towards Human-Level Text-to-Speech" (2023)
- "When Humans Growl and Birds Speak" (arXiv 2505.24336)
- "F5-TTS: Fairytaler that Fakes Fluent Speech with Flow Matching"

### Repositories
- DDSP: https://github.com/magenta/ddsp
- StyleTTS2: https://github.com/yl4579/StyleTTS2
- F5-TTS: https://github.com/SWivid/F5-TTS
- Bark: https://github.com/suno-ai/bark
- Fish Speech: https://github.com/fishaudio/fish-speech
- OpenVoice: https://github.com/myshell-ai/OpenVoice

### Commercial APIs (Backup)
- ElevenLabs Voice Design: https://elevenlabs.io/voice-cloning
- Respeecher (film-grade): https://www.respeecher.com

---

## Success Criteria

1. **Recognizability**: Voice is distinctly "Caro" - different from generic TTS
2. **Naturalness**: MOS (Mean Opinion Score) > 3.5 for intelligibility
3. **Character**: Subtle dog-derived characteristics audible but not distracting
4. **Performance**: Generation < 500ms for short phrases
5. **Integration**: Works with Caro CLI architecture

---

## Next Steps

1. [ ] Record Kyaro audio samples (5-15 minutes)
2. [ ] Set up Python environment with audio tools
3. [ ] Extract and analyze Kyaro's vocal features
4. [ ] Prototype with DDSP timbre transfer
5. [ ] Test StyleTTS2 with Kyaro style vectors
6. [ ] Select best approach and refine
7. [ ] Create Caro voice model
8. [ ] Integrate with CLI (audio clips or TTS)

---

## Appendix: Kyaro's Vocal Characteristics to Capture

Based on dog vocalization research, we want to extract:

| Characteristic | Acoustic Feature | Human Voice Mapping |
|----------------|------------------|---------------------|
| Alertness | Higher F0, sharp onset | Energetic, clear attack |
| Warmth | Lower F2, broader formants | Warm, friendly tone |
| Playfulness | Variable pitch, harmonic richness | Expressive prosody |
| Confidence | Strong fundamental, high SNR | Clear, authoritative |

The goal is a voice that sounds like a helpful, friendly assistant with a unique "Kyaro-derived" personality.
