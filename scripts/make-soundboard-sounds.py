"""Regenerate the soundboard's built-in sounds in static/soundboard/.

Every sound is synthesized here from noise and sine/pluck tones, so the
files carry no third-party rights (released CC0 with Catalyst). Needs numpy
and ffmpeg: python scripts/make-soundboard-sounds.py
"""

import subprocess
import tempfile
import wave
from pathlib import Path

import numpy as np

SR = 32000
OUT = Path(__file__).resolve().parent.parent / "static" / "soundboard"
rng = np.random.default_rng(7)


def t(seconds):
    return np.arange(int(seconds * SR)) / SR


def lowpass(x, cutoff):
    a = np.exp(-2 * np.pi * cutoff / SR)
    y = np.empty_like(x)
    acc = 0.0
    for i, v in enumerate(x):
        acc = (1 - a) * v + a * acc
        y[i] = acc
    return y


def bandpass(x, lo, hi):
    return lowpass(x, hi) - lowpass(x, lo)


def env(n, attack, decay):
    k = np.arange(n) / SR
    return np.minimum(1, k / max(attack, 1e-4)) * np.exp(-k / decay)


def pluck(freq, seconds, decay=0.35):
    k = t(seconds)
    tone = sum(np.sin(2 * np.pi * freq * h * k) / h**1.5 for h in range(1, 6))
    return tone * env(len(k), 0.004, decay)


def place(buf, sound, at):
    i = int(at * SR)
    j = min(len(buf), i + len(sound))
    buf[i:j] += sound[: j - i]


def seamless(x, fade=1.0):
    """Crossfade the tail into the head so the file loops without a click."""
    n = int(fade * SR)
    ramp = np.linspace(0, 1, n)
    head = x[:n] * ramp + x[-n:] * (1 - ramp)
    return np.concatenate([head, x[n:-n]])


def note(name):
    names = {"C": -9, "D": -7, "E": -5, "F": -4, "G": -2, "A": 0, "B": 2}
    semis = names[name[0]] + (1 if "#" in name else 0) + 12 * (int(name[-1]) - 4)
    return 440 * 2 ** (semis / 12)


def tavern():
    beat = 60 / 132
    melody = "D5 F#5 A5 F#5 G5 E5 C#5 E5 D5 F#5 A5 D6 C#6 A5 E5 C#5".split()
    bass = "D3 D3 A2 A2 G2 G2 A2 A2".split()
    buf = np.zeros(int((len(melody) * 2 * beat + 1.2) * SR))
    for rep in range(2):
        for i, n in enumerate(melody):
            place(buf, 0.32 * pluck(note(n), 0.9), (rep * len(melody) + i) * beat)
        for i, n in enumerate(bass):
            place(buf, 0.4 * pluck(note(n), 1.6, 0.6), rep * len(melody) * beat + i * 2 * beat)
    return seamless(buf, 1.2)


def dungeon():
    k = t(25)
    chord = [note("A2"), note("C3"), note("E3"), note("A1")]
    x = sum(np.sin(2 * np.pi * f * k + np.sin(2 * np.pi * 0.07 * k * (i + 1))) for i, f in enumerate(chord))
    x *= 0.6 + 0.4 * np.sin(2 * np.pi * k / 12.5)
    x += 0.15 * lowpass(rng.standard_normal(len(k)), 300)
    return seamless(0.18 * x, 3)


def drums():
    beat = 60 / 140
    buf = np.zeros(int((32 * beat + 0.8) * SR))
    for i in range(32):
        k = t(0.6)
        boom = np.sin(2 * np.pi * (60 + 80 * np.exp(-k * 25)) * k) * env(len(k), 0.002, 0.18)
        if i % 4 in (0, 3) or i % 8 == 6:
            place(buf, 0.9 * boom, i * beat)
        if i % 2 == 1:
            hit = bandpass(rng.standard_normal(int(0.15 * SR)), 1500, 6000) * env(int(0.15 * SR), 0.001, 0.04)
            place(buf, 0.6 * hit, i * beat)
    return seamless(buf, 0.8)


def rain():
    n = int(26 * SR)
    x = 0.5 * lowpass(rng.standard_normal(n), 3000) - 0.3 * lowpass(rng.standard_normal(n), 200)
    for at in rng.uniform(0, 25.9, 900):
        drop = bandpass(rng.standard_normal(int(0.03 * SR)), 2000, 7000) * env(int(0.03 * SR), 0.0005, 0.006)
        place(x, rng.uniform(0.2, 0.6) * drop, at)
    return seamless(0.6 * x, 2)


def wind():
    n = int(26 * SR)
    k = t(26)
    gust = 0.55 + 0.45 * np.sin(2 * np.pi * k / 13 + 0.6 * np.sin(2 * np.pi * k / 6.5))
    x = bandpass(rng.standard_normal(n), 250, 900) * gust
    return seamless(1.6 * x, 3)


def campfire():
    n = int(26 * SR)
    x = 0.8 * lowpass(lowpass(rng.standard_normal(n), 120), 120)
    for at in rng.uniform(0, 25.9, 700):
        m = int(rng.uniform(0.002, 0.012) * SR)
        crack = rng.standard_normal(m) * env(m, 0.0002, 0.002)
        place(x, rng.uniform(0.2, 1.0) * crack, at)
    return seamless(0.5 * x, 2)


def sword():
    k = t(1.6)
    partials = [1210, 1873, 2640, 3399, 4720]
    ring = sum(np.sin(2 * np.pi * f * k) * np.exp(-k * (2 + i)) for i, f in enumerate(partials)) / 3
    hit = rng.standard_normal(len(k)) * env(len(k), 0.0005, 0.02)
    return 0.6 * (ring + hit)


def thunder():
    n = int(6 * SR)
    k = t(6)
    crack = bandpass(rng.standard_normal(n), 400, 5000) * env(n, 0.005, 0.25)
    rumble = lowpass(lowpass(rng.standard_normal(n), 90), 90) * 18 * env(n, 0.3, 1.8)
    rumble *= 0.7 + 0.3 * np.sin(2 * np.pi * 1.3 * k)
    return 0.5 * (crack + rumble)


def chime():
    buf = np.zeros(int(2.8 * SR))
    for i, n in enumerate(["E5", "G#5", "B5", "E6", "G#6"]):
        k = t(2.2)
        bell = (np.sin(2 * np.pi * note(n) * k) + 0.4 * np.sin(2 * np.pi * note(n) * 2.76 * k)) * env(len(k), 0.002, 0.7)
        place(buf, 0.25 * bell, i * 0.09)
    return buf


def march():
    beat = 60 / 108
    melody = "G4 B4 D5 B4 C5 A4 F#4 A4 G4 B4 D5 G5 F#5 D5 A4 F#4".split()
    buf = np.zeros(int((len(melody) * 2 * beat + 1.2) * SR))
    for rep in range(2):
        start = rep * len(melody) * beat
        for i, n in enumerate(melody):
            place(buf, 0.3 * pluck(note(n), 1.0, 0.45), start + i * beat)
        for i in range(len(melody)):
            k = t(0.4)
            step = np.sin(2 * np.pi * (70 + 40 * np.exp(-k * 30)) * k) * env(len(k), 0.002, 0.1)
            place(buf, 0.5 * step, start + i * beat)
            place(buf, 0.25 * pluck(note("G2" if i % 4 < 2 else "D2"), 0.8, 0.4), start + i * beat)
    return seamless(buf, 1.2)


def suspense():
    k = t(24)
    low = sum(np.sin(2 * np.pi * f * k) for f in (note("C2"), note("C#2"), note("F#2")))
    trem = 0.6 + 0.4 * np.sin(2 * np.pi * 7 * k)
    x = 0.2 * low * trem * (0.5 + 0.5 * np.sin(2 * np.pi * k / 12))
    for at in np.arange(0, 24, 1.1):
        for off, amp in ((0, 1.0), (0.22, 0.7)):
            kk = t(0.25)
            thump = np.sin(2 * np.pi * 50 * kk) * env(len(kk), 0.003, 0.06)
            place(x, 0.8 * amp * thump, at + off)
    return seamless(x, 2)


def storm():
    x = rain()
    for at in (4.0, 13.5, 20.0):
        place(x, 0.5 * thunder()[: int(5 * SR)], at)
    return seamless(x, 1)


def forest():
    n = int(26 * SR)
    x = 0.6 * bandpass(rng.standard_normal(n), 300, 1200) * (0.7 + 0.3 * np.sin(2 * np.pi * t(26) / 13))
    for at in rng.uniform(0, 25, 70):
        f0 = rng.uniform(2500, 4500)
        for c in range(rng.integers(2, 5)):
            k = t(0.12)
            chirp = np.sin(2 * np.pi * (f0 + 1500 * k / 0.12) * k) * env(len(k), 0.01, 0.04)
            place(x, 0.25 * chirp, at + c * 0.15)
    return seamless(x, 2)


def cave():
    n = int(26 * SR)
    x = 0.5 * lowpass(lowpass(rng.standard_normal(n), 80), 80)
    for at in rng.uniform(0, 25.5, 40):
        k = t(0.5)
        f = rng.uniform(900, 1600)
        drip = np.sin(2 * np.pi * f * (1 + 0.6 * np.exp(-k * 40)) * k) * env(len(k), 0.001, 0.05)
        place(x, 0.35 * drip, at)
        place(x, 0.12 * drip, at + 0.23)
    return seamless(x, 2)


def night_camp():
    x = campfire()
    k = t(len(x) / SR)
    crickets = np.sin(2 * np.pi * 4700 * k) * (np.sin(2 * np.pi * 30 * k) > 0.3) * (np.sin(2 * np.pi * 0.8 * k) > 0)
    return x + 0.05 * crickets


def ocean():
    n = int(24 * SR)
    k = t(24)
    swell = np.clip(np.sin(2 * np.pi * k / 8), 0, None) ** 2
    x = lowpass(rng.standard_normal(n), 1200) * (0.2 + swell)
    x += 0.6 * lowpass(lowpass(rng.standard_normal(n), 60), 60)
    return seamless(1.2 * x, 2)


def spell():
    k = t(1.8)
    sweep = np.sin(2 * np.pi * (300 + 900 * k**2) * k) * env(len(k), 0.3, 0.8)
    whoosh = bandpass(rng.standard_normal(len(k)), 800, 4000) * env(len(k), 0.4, 0.5)
    return 0.5 * sweep + 0.6 * whoosh


def creak():
    k = t(2.0)
    f = 140 + 60 * np.sin(2 * np.pi * 0.4 * k) + 15 * rng.standard_normal(len(k)).cumsum() / SR * 50
    phase = 2 * np.pi * np.cumsum(f) / SR
    saw = (phase / (2 * np.pi)) % 1 - 0.5
    pulses = (np.sin(phase * 0.5) > 0.6).astype(float)
    return bandpass(saw * pulses, 200, 2500) * np.sin(np.pi * k / 2.0)


def growl():
    k = t(2.4)
    f = 55 + 10 * np.sin(2 * np.pi * 3 * k)
    phase = 2 * np.pi * np.cumsum(f) / SR
    tone = np.sign(np.sin(phase)) * (1 + 0.5 * rng.standard_normal(len(k)))
    return lowpass(lowpass(tone, 500), 700) * np.sin(np.pi * k / 2.4) ** 0.5


def eerie():
    k = t(26)
    x = np.zeros(len(k))
    for i, f in enumerate((note("D#3"), note("A3"), note("D4"), note("G#4"))):
        drift = 1 + 0.004 * np.sin(2 * np.pi * k / (9 + 3 * i))
        x += np.sin(2 * np.pi * f * np.cumsum(drift) / SR) * (0.5 + 0.5 * np.sin(2 * np.pi * k / (13 / (i + 1)) + i))
    x += 0.4 * bandpass(rng.standard_normal(len(k)), 2000, 5000) * (0.5 + 0.5 * np.sin(2 * np.pi * k / 6.5))
    return seamless(0.2 * x, 3)


def tension():
    k = t(24)
    x = sum(np.sin(2 * np.pi * f * k) for f in (note("C2"), note("G2"), note("C#3")))
    x *= 0.7 + 0.3 * np.sin(2 * np.pi * 2 * k)
    for at in np.arange(0, 24, 0.75):
        tick = bandpass(rng.standard_normal(int(0.05 * SR)), 3000, 8000) * env(int(0.05 * SR), 0.001, 0.01)
        place(x, 0.6 * tick, at)
    return seamless(0.25 * x, 2)


def epic_swell():
    k = t(24)
    rise = np.sin(np.pi * (k % 12) / 12) ** 2
    chord = (note("D2"), note("A2"), note("D3"), note("F3"), note("A3"), note("D4"))
    x = sum(np.sin(2 * np.pi * f * k) + 0.3 * np.sin(2 * np.pi * 2 * f * k) for f in chord) * rise
    x += 0.5 * lowpass(rng.standard_normal(len(k)), 600) * rise
    return seamless(0.15 * x, 3)


SOUNDS = {
    "tavern-jig": tavern,
    "travel-march": march,
    "dungeon-drone": dungeon,
    "battle-drums": drums,
    "suspense": suspense,
    "rain-storm": storm,
    "wind": wind,
    "forest": forest,
    "cave": cave,
    "night-camp": night_camp,
    "ocean": ocean,
    "eerie-atmosphere": eerie,
    "tension-bed": tension,
    "epic-swell": epic_swell,
    "sword-clash": sword,
    "spell-cast": spell,
    "thunder": thunder,
    "magic-chime": chime,
    "door-creak": creak,
    "monster-growl": growl,
}


def write(name, x):
    # Even loudness across pads: aim for -18 dBFS RMS, never past -1 dBFS peak.
    x = x * min(0.126 / max(1e-9, np.sqrt(np.mean(x**2))), 0.89 / max(1e-9, np.max(np.abs(x))))
    with tempfile.TemporaryDirectory() as tmp:
        wav = Path(tmp) / "s.wav"
        with wave.open(str(wav), "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(SR)
            w.writeframes((x * 32767).astype("<i2").tobytes())
        subprocess.run(
            ["ffmpeg", "-y", "-loglevel", "error", "-i", str(wav), "-c:a", "libvorbis", "-q:a", "2",
             "-map_metadata", "-1", str(OUT / f"{name}.ogg")],
            check=True,
        )


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    for name, make in SOUNDS.items():
        write(name, make())
        print(name)
