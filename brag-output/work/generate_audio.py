import os
import wave
import struct
import numpy as np

def generate_soundtrack(output_path, duration=20.0, sample_rate=44100):
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    n_samples = int(duration * sample_rate)
    t = np.linspace(0, duration, n_samples, endpoint=False)
    
    # 120 BPM: beat = 0.5s (2 beats per second)
    beat_dur = 0.5
    total_beats = int(duration / beat_dur)
    
    audio_l = np.zeros(n_samples)
    audio_r = np.zeros(n_samples)
    
    # 1. 808 Kick Drum (Every Beat 0, 2 or downbeats)
    for b in range(total_beats):
        bt = b * beat_dur
        b_idx = int(bt * sample_rate)
        # Kick on every beat, stronger on downbeats (b % 4 == 0 or 2)
        if b % 2 == 0 or b >= 24:
            k_dur = 0.25
            k_samples = int(k_dur * sample_rate)
            kt = np.linspace(0, k_dur, k_samples, endpoint=False)
            # Pitch drop from 140Hz down to 42Hz
            freq = 42 + 98 * np.exp(-kt * 22)
            phase = 2 * np.pi * np.cumsum(freq) / sample_rate
            env = np.exp(-kt * 14)
            kick = np.sin(phase) * env * 0.45
            
            end_idx = min(n_samples, b_idx + k_samples)
            actual_len = end_idx - b_idx
            audio_l[b_idx:end_idx] += kick[:actual_len]
            audio_r[b_idx:end_idx] += kick[:actual_len]
            
    # 2. Snare / Claps on Beats 1 & 3 (b % 2 == 1)
    for b in range(total_beats):
        if b % 2 == 1:
            bt = b * beat_dur
            b_idx = int(bt * sample_rate)
            s_dur = 0.18
            s_samples = int(s_dur * sample_rate)
            st = np.linspace(0, s_dur, s_samples, endpoint=False)
            # Noise + tone burst
            noise = np.random.uniform(-1, 1, s_samples)
            tone = np.sin(2 * np.pi * 195 * st) * 0.4
            env = np.exp(-st * 28)
            snare = (noise * 0.65 + tone * 0.35) * env * 0.28
            
            end_idx = min(n_samples, b_idx + s_samples)
            actual_len = end_idx - b_idx
            audio_l[b_idx:end_idx] += snare[:actual_len] * 0.95
            audio_r[b_idx:end_idx] += snare[:actual_len] * 1.05

    # 3. Metallic 16th-note Hi-Hats (every 0.125s)
    hat_dur = 0.04
    hat_samples = int(hat_dur * sample_rate)
    ht = np.linspace(0, hat_dur, hat_samples, endpoint=False)
    hat_env = np.exp(-ht * 85)
    for i in range(int(duration / 0.125)):
        ht_start = i * 0.125
        h_idx = int(ht_start * sample_rate)
        if h_idx + hat_samples < n_samples:
            hat = np.random.uniform(-1, 1, hat_samples) * hat_env * 0.08
            pan = 0.8 if i % 2 == 0 else 1.2
            audio_l[h_idx:h_idx+hat_samples] += hat * pan
            audio_r[h_idx:h_idx+hat_samples] += hat * (2.0 - pan)

    # 4. Driving Synth Bassline (D minor: D2, F2, G2, A2)
    # Notes: D2=73.4Hz, F2=87.3Hz, G2=98.0Hz, A2=110.0Hz
    bass_prog = [73.41, 73.41, 87.31, 98.0, 73.41, 73.41, 110.0, 98.0]
    for b in range(total_beats):
        note_freq = bass_prog[b % len(bass_prog)]
        bt = b * beat_dur
        b_idx = int(bt * sample_rate)
        note_dur = 0.42
        n_sub_samples = int(note_dur * sample_rate)
        nt = np.linspace(0, note_dur, n_sub_samples, endpoint=False)
        # Sawtooth harmonic rich wave
        saw = (2 * (nt * note_freq - np.floor(0.5 + nt * note_freq))) * 0.5
        sine = np.sin(2 * np.pi * note_freq * nt) * 0.5
        env = np.exp(-nt * 5)
        bass = (saw * 0.5 + sine * 0.5) * env * 0.32
        
        end_idx = min(n_samples, b_idx + n_sub_samples)
        actual_len = end_idx - b_idx
        audio_l[b_idx:end_idx] += bass[:actual_len]
        audio_r[b_idx:end_idx] += bass[:actual_len]

    # 5. Ambient Atmospheric Chord Arp (D3=146.8, F3=174.6, A3=220.0, C4=261.6)
    chord_freqs = [146.83, 174.61, 220.00, 261.63, 293.66, 349.23]
    for i in range(int(duration / 0.25)):
        freq = chord_freqs[i % len(chord_freqs)]
        t_start = i * 0.25
        c_idx = int(t_start * sample_rate)
        c_dur = 0.35
        c_samples = int(c_dur * sample_rate)
        ct = np.linspace(0, c_dur, c_samples, endpoint=False)
        tone = np.sin(2 * np.pi * freq * ct) * np.exp(-ct * 7) * 0.12
        end_idx = min(n_samples, c_idx + c_samples)
        actual_len = end_idx - c_idx
        pan_l = 0.6 + 0.4 * np.sin(i * 0.7)
        pan_r = 1.4 - pan_l
        audio_l[c_idx:end_idx] += tone[:actual_len] * pan_l
        audio_r[c_idx:end_idx] += tone[:actual_len] * pan_r

    # 6. Scene Transition Impacts & Hits (At 0.0s, 3.0s, 7.0s, 12.0s, 16.0s)
    hit_times = [0.0, 3.0, 7.0, 12.0, 16.0]
    for ht in hit_times:
        h_idx = int(ht * sample_rate)
        h_dur = 0.8
        h_samples = int(h_dur * sample_rate)
        ht_arr = np.linspace(0, h_dur, h_samples, endpoint=False)
        sub = np.sin(2 * np.pi * 55 * np.exp(-ht_arr * 4) * ht_arr) * np.exp(-ht_arr * 5) * 0.5
        noise = np.random.uniform(-1, 1, h_samples) * np.exp(-ht_arr * 12) * 0.3
        impact = sub + noise
        end_idx = min(n_samples, h_idx + h_samples)
        actual_len = end_idx - h_idx
        audio_l[h_idx:end_idx] += impact[:actual_len]
        audio_r[h_idx:end_idx] += impact[:actual_len]

    # 7. Keystroke Ticks during Scene 3 (7.2s to 9.5s)
    key_times = np.linspace(7.2, 9.4, 28)
    for kt in key_times:
        k_idx = int(kt * sample_rate)
        kd_samples = int(0.015 * sample_rate)
        if k_idx + kd_samples < n_samples:
            click = np.random.uniform(-1, 1, kd_samples) * 0.1
            audio_l[k_idx:k_idx+kd_samples] += click
            audio_r[k_idx:k_idx+kd_samples] += click

    # 8. Synth Riser (14.8s to 16.0s leading to Outro)
    riser_start = 14.8
    riser_dur = 1.2
    r_idx = int(riser_start * sample_rate)
    r_samples = int(riser_dur * sample_rate)
    rt = np.linspace(0, riser_dur, r_samples, endpoint=False)
    riser_freq = 110 * np.exp(rt * 1.8) # 110Hz to ~900Hz
    riser = np.sin(2 * np.pi * np.cumsum(riser_freq) / sample_rate) * (rt / riser_dur) * 0.22
    end_idx = min(n_samples, r_idx + r_samples)
    actual_len = end_idx - r_idx
    audio_l[r_idx:end_idx] += riser[:actual_len]
    audio_r[r_idx:end_idx] += riser[:actual_len]

    # Master Limiter & Normalize (-1.0 dB ceiling)
    max_val = max(np.max(np.abs(audio_l)), np.max(np.abs(audio_r)))
    if max_val > 0:
        scale = 0.88 / max_val
        audio_l *= scale
        audio_r *= scale

    # Write 16-bit PCM WAV
    with wave.open(output_path, 'wb') as wav:
        wav.setnchannels(2)
        wav.setsampwidth(2)
        wav.setframerate(sample_rate)
        
        # Interleave stereo channels
        interleaved = np.empty((n_samples * 2,), dtype=np.int16)
        interleaved[0::2] = (audio_l * 32767).astype(np.int16)
        interleaved[1::2] = (audio_r * 32767).astype(np.int16)
        wav.writeframes(interleaved.tobytes())

    print(f"Soundtrack generated: {output_path} ({duration}s @ {sample_rate}Hz)")

if __name__ == "__main__":
    generate_soundtrack("D:/PROJECTS/LoomDB/brag-output/work/soundtrack.wav")
