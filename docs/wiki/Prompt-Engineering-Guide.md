# 🎙️ Master Class: Prompt Engineering for Audio

Generative audio requires a shift in mindset from text models. Instead of describing *what happens*, you must describe the **sonic texture, spatial environment, and emotional resonance** of the target sound.

---

## 🎲 Advanced Soundscapes (`generate_soundscape`)

### Ensuring Vocal-Free Environments
By default, the server appends aggressive negative prompts to ensure instrumental-only output. However, your prompt should reinforce this to avoid unwanted vocal "artifacts" or humming.

- **❌ Ambiguous**: "A rainy street in Japan."
- **✅ Sonic Precision**: "Heavy rhythmic rain hitting a glass window in a futuristic cyberpunk city. Drip-drip textures, muffled asphalt puddles. **Pure instrumental foley, no voices, no music.** Deep low-frequency hover-car hums, binaural spatial audio."

---

## 🎵 Professional Music Production (`generate_music`)

### Structural Composition with Timestamps
The Lyria 3 model understands temporal cues. Use `[mm:ss - mm:ss]` syntax within your prompt to dictate section changes, instrumentation shifts, and dynamic evolution.

> **Example**: "Lo-fi jazz hip hop, 85 BPM. **[00:00 - 00:30]** Sparse mellow Rhodes piano chords. **[00:30 - 01:00]** Dusty vinyl crackle and a boom-bap drum loop enter. **[01:00 - 02:00]** A melancholic muted trumpet melody leads."

### Metadata Keywords
The server "bakes" these keywords into the final prompt. Use them to maintain technical consistency across multiple generations:
- **BPM**: Target tempo (e.g., `BPM: 120`).
- **Key**: Harmonic center (e.g., `Key: A-flat major`).
- **Intensity**: Dynamic energy level (e.g., `Intensity: 8/10`).

---

## 🗣️ Expressive Voice & Narration (`generate_voice`)

The Gemini 2.5 Native Audio model is highly sensitive to **Voice Direction**. Instead of just providing text, describe the *character's state* and *vocal technique*.

- **Character Profile**: "A raspy, ancient wizard with a dry cough. Read slowly with long pauses."
- **Emotional Arc**: "Start whispering in a terrified voice, then suddenly shout with a high-pitched frantic tone."
- **Acoustic Environment**: "Read the text as if the speaker is inside a large, empty metal tank with a 4-second echoing reverb."

---

## 🔊 Sound Effects & Foley (`generate_sfx`)

For one-shot SFX, brevity and "isolation" are key. Use punchy, descriptive adjectives.

- **Bad**: "A laser beam shooting."
- **Good**: "A high-pitched metallic 'PEW' laser blast. Synthesized pulse wave, crisp transients, isolated sound effect, zero background music."

---

## 🖼 Multimodal Guidance (The `image_path` Parameter)

One of the most powerful features of the Gemini Audio MCP is the ability to guide audio with visual input.
1. **Texture Mapping**: An image of a rough stone wall will influence the model to generate audio with "gritty" and "damped" acoustic properties.
2. **Atmosphere Synchronization**: Provide a screenshot of your game level (e.g., a neon cyberpunk alley) to synchronize the generation's "mood" with the visual aesthetic.

---

## 🛠 Pro-Tips for Prompt Design

1. **The "Layering" Method**: Describe sounds from the background to the foreground. (e.g., "Distant city hum [background] -> Muffled chatter [midground] -> Sharp glass breaking [foreground]").
2. **Suppression Phrases**: Use "pure", "isolated", "dry", and "no-reverb" to get clean, manipulatable assets.
3. **Intensity Tuning**: For music, if the model is too chaotic, lower the `intensity` parameter to `2/10` to get a sparse, minimal arrangement.
