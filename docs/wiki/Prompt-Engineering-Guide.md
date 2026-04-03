# 🎙️ Prompt Engineering for Audio

Getting high-quality results from Gemini and Lyria requires a slightly different approach than text generation. Audio models "listen" for descriptive cues about texture, space, and emotion.

## 🎲 Soundscape Generation (`generate_soundscape`)
*Best for: Environmental ambience, background textures.*

The Gemini 2.0 Live API excels at complex, layered sounds. Use "sensory" adjectives.

*   **❌ Bad**: "rain in a city"
*   **✅ Good**: "Heavy rhythmic rain hitting a glass window in a futuristic cyberpunk city, distant low-frequency hovercar hums, neon signs buzzing faintly, binaural spatial audio."

**Tip: The "Seamless Loop" trick**
When generating loops, explicitly tell the model: *"Constant background texture with no sudden peaks, suitable for a seamless loop."*

---

## 🗣️ Voice Generation (`generate_voice`)
*Best for: Narration, character dialogue, expressive reading.*

The Native Audio model can follow "voice direction" within the prompt.

*   **Direction**: "Read the following in a raspy, old wizard's voice, pausing for dramatic effect after every sentence."
*   **Emotion**: "A nervous, high-pitched voice that cracks slightly when speaking about the monster."

---

## 🎵 Music & SFX (`generate_music` / `generate_sfx`)
*Powered by Lyria 3 (Pro/Clip).*

Lyria responds well to genre, instrumentation, and BPM.

*   **Genre-based**: "Lo-fi hip hop with a dusty vinyl crackle, mellow electric piano, 85 BPM, relaxing study vibes."
*   **Instrumentation**: "Solo cello playing a melancholic minor-key melody in a large cathedral with long reverb."
*   **SFX**: "A crisp, metallic 'ching' sound effect for a level-up notification, high-pitched and rewarding."
