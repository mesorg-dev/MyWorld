use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use std::f32::consts::TAU;

#[derive(Clone, Debug)]
pub struct SoundBank {
    strike: Sound,
    knife: Sound,
    dash: Sound,
    time_stop: Sound,
    impact: Sound,
    roller: Sound,
}

impl SoundBank {
    pub async fn load() -> Result<Self, macroquad::Error> {
        Ok(Self {
            strike: load_sound_from_bytes(&tone_wav(180.0, 0.09, 55.0)).await?,
            knife: load_sound_from_bytes(&tone_wav(720.0, 0.07, -240.0)).await?,
            dash: load_sound_from_bytes(&tone_wav(360.0, 0.12, -180.0)).await?,
            time_stop: load_sound_from_bytes(&tone_wav(92.0, 0.34, -42.0)).await?,
            impact: load_sound_from_bytes(&tone_wav(125.0, 0.13, -60.0)).await?,
            roller: load_sound_from_bytes(&tone_wav(68.0, 0.48, -28.0)).await?,
        })
    }

    pub fn strike(&self, enabled: bool) {
        play(&self.strike, enabled, 0.22);
    }

    pub fn knife(&self, enabled: bool) {
        play(&self.knife, enabled, 0.16);
    }

    pub fn dash(&self, enabled: bool) {
        play(&self.dash, enabled, 0.18);
    }

    pub fn time_stop(&self, enabled: bool) {
        play(&self.time_stop, enabled, 0.28);
    }

    pub fn impact(&self, enabled: bool) {
        play(&self.impact, enabled, 0.26);
    }

    pub fn roller(&self, enabled: bool) {
        play(&self.roller, enabled, 0.34);
    }
}

fn play(sound: &Sound, enabled: bool, volume: f32) {
    if enabled {
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume,
            },
        );
    }
}

fn tone_wav(frequency: f32, duration: f32, sweep: f32) -> Vec<u8> {
    const SAMPLE_RATE: u32 = 22_050;
    let sample_count = (SAMPLE_RATE as f32 * duration) as usize;
    let data_size = (sample_count * 2) as u32;
    let mut wav = Vec::with_capacity(data_size as usize + 44);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    for index in 0..sample_count {
        let time = index as f32 / SAMPLE_RATE as f32;
        let envelope = (1.0 - time / duration).max(0.0).powf(2.2);
        let phase = TAU * (frequency * time + sweep * time * time * 0.5);
        let sample = (phase.sin() * envelope * 0.42 * i16::MAX as f32) as i16;
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tones_have_valid_pcm_wave_headers_and_payloads() {
        let wav = tone_wav(440.0, 0.1, 0.0);
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(
            u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize,
            wav.len() - 44
        );
    }
}
