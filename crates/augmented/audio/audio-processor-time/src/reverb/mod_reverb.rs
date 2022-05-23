// Augmented Audio: Audio libraries and applications
// Copyright (c) 2022 Pedro Tacla Yamada
//
// The MIT License (MIT)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::time::Duration;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use audio_garbage_collector::{make_shared, Shared};
use audio_processor_traits::parameters::{
    make_handle_ref, AudioProcessorHandle, AudioProcessorHandleProvider, AudioProcessorHandleRef,
    ParameterSpec, ParameterValue,
};
use audio_processor_traits::{
    AudioBuffer, AudioProcessor, AudioProcessorSettings, SimpleAudioProcessor,
};
use augmented_dsp_filters::rbj::{FilterProcessor, FilterType};
use augmented_oscillator::Oscillator;

use crate::MonoDelayProcessor;

#[allow(dead_code)]
fn shuffle(rng: &[usize], frame: &mut [f32]) {
    for i in 0..frame.len() {
        frame[i] = frame[rng[i]];
    }
}

fn flip_polarities(frame: &mut [f32]) {
    for i in 0..frame.len() {
        frame[i] = -frame[i];
    }
}

fn hadamard_matrix(frame: &mut [f32]) {
    let matrix = nalgebra::Matrix4::new(
        1.0, 1.0, 1.0, 1.0, // \n
        1.0, -1.0, 1.0, -1.0, // \n
        1.0, 1.0, -1.0, -1.0, // \n
        1.0, -1.0, -1.0, 1.0,
    );
    let target = nalgebra::Matrix1x4::new(frame[0], frame[1], frame[2], frame[3]);
    let result = target * matrix;
    for (r, slot) in result.iter().zip(frame) {
        *slot = *r;
    }
}

fn householder(frame: &mut [f32]) {
    let matrix = nalgebra::Matrix4::new(
        1.0, -1.0, -1.0, -1.0, // \n
        -1.0, 1.0, -1.0, -1.0, // \n
        -1.0, -1.0, 1.0, -1.0, // \n
        -1.0, -1.0, -1.0, 1.0,
    );
    let target = nalgebra::Matrix1x4::new(frame[0], frame[1], frame[2], frame[3]);
    let result = target * matrix;
    for (r, slot) in result.iter().zip(frame) {
        *slot = *r;
    }
}

struct ModReverbHandle {}

/// Implements the reverb described by Geraint Luff on:
/// "Let's write a Reverb - ADC21 - https://www.youtube.com/watch?v=6ZK2Goiyotk"
pub struct ModReverbProcessor {
    handle: Shared<ModReverbHandle>,
    diffusers: [Diffuser; 6],
    delay: [MonoDelayProcessor<f32>; 8],
    filter: [FilterProcessor<f32>; 2],
    diffuser_modulator: Oscillator<f32>,
    delay_modulator: Oscillator<f32>,
}

struct GenericHandle(Shared<ModReverbHandle>);

impl AudioProcessorHandleProvider for ModReverbProcessor {
    fn generic_handle(&self) -> AudioProcessorHandleRef {
        make_handle_ref(GenericHandle(self.handle.clone()))
    }
}

impl AudioProcessorHandle for GenericHandle {
    fn parameter_count(&self) -> usize {
        0
    }

    fn get_parameter_spec(&self, _index: usize) -> ParameterSpec {
        todo!()
    }

    fn get_parameter(&self, _index: usize) -> Option<ParameterValue> {
        todo!()
    }

    fn set_parameter(&self, _index: usize, _request: ParameterValue) {
        todo!()
    }
}

impl Default for ModReverbProcessor {
    fn default() -> Self {
        Self {
            handle: make_shared(ModReverbHandle {}),
            diffusers: [
                Diffuser::default(),
                Diffuser::default(),
                Diffuser::default(),
                Diffuser::default(),
                Diffuser::default(),
                Diffuser::default(),
            ],
            delay: [
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
            ],
            filter: [
                FilterProcessor::new(FilterType::LowPass),
                FilterProcessor::new(FilterType::LowPass),
            ],
            diffuser_modulator: Oscillator::sine(44100.0),
            delay_modulator: Oscillator::sine(44100.0),
        }
    }
}

impl AudioProcessor for ModReverbProcessor {
    type SampleType = f32;

    fn prepare(&mut self, settings: AudioProcessorSettings) {
        let mut max_delay_time = 0.4 / (self.diffusers.len() as f32).powf(2.0);
        for diffuser in self.diffusers.iter_mut() {
            diffuser.max_delay_time = Duration::from_secs_f32(max_delay_time);
            diffuser.prepare(settings);
            max_delay_time *= 2.0;
        }

        for delay in &mut self.delay {
            delay.s_prepare(settings);
            delay.handle().set_delay_time_secs(0.2);
            delay.handle().set_feedback(0.1)
        }

        self.diffuser_modulator
            .set_sample_rate(settings.sample_rate());
        self.diffuser_modulator.set_frequency(0.4);
        self.delay_modulator.set_sample_rate(settings.sample_rate());
        self.delay_modulator.set_frequency(0.6);

        for filter in &mut self.filter {
            filter.s_prepare(settings);
            filter.set_q(1.0);
            filter.set_cutoff(800.0);
        }
    }

    fn process<BufferType: AudioBuffer<SampleType = Self::SampleType>>(
        &mut self,
        data: &mut BufferType,
    ) {
        // Last delay line feedback / volume
        let delay_feedback = 0.2;
        let delay_volume = 0.1;
        // Reverb volume
        let reverb_volume = 0.7;

        // For each frame
        for frame in data.frames_mut() {
            // Modulate diffusion delay times
            let diffuser_modulation = self.diffuser_modulator.next_sample(); // -1.0..1.0
            let diffuser_modulation = 1.0 + diffuser_modulation * 0.0005;
            for diffuser in self.diffusers.iter_mut() {
                diffuser.set_delay_mult(diffuser_modulation);
            }
            // Modulate multi-channel delay times
            let delay_modulation = self.delay_modulator.next_sample();
            let delay_modulation = 1.0 + delay_modulation * 0.01;
            for delay in &mut self.delay {
                let duration = 0.2 * delay_modulation;
                delay.handle().set_delay_time_secs(duration);
            }

            let left = frame[0];
            let right = frame[1];

            // Generate a 8 channel input signal
            let mut frame8 = [
                frame[0], frame[1], frame[0], frame[1], frame[0], frame[1], frame[0], frame[1],
            ];

            // Run it through a diffusion step
            for diffuser in &mut self.diffusers {
                diffuser.process(&mut frame8);
            }

            // Run it through a multi-channel delay line
            let mut delayed = [0.0, 0.0, 0.0, 0.0];
            for (delay, delay_output) in self.delay.iter_mut().zip(&mut delayed) {
                *delay_output = delay.read();
            }

            // Shuffle the channels together
            householder(&mut delayed);

            // Write back into the multi-channel delay line and generate output
            for ((sample, delay), delay_output) in
                frame8.iter_mut().zip(&mut self.delay).zip(delayed)
            {
                delay.write(*sample + delay_output * delay_feedback);
                *sample = *sample + delay_output * delay_volume;
            }

            // Mix the multi-channel output back into stereo
            let scale = 1.0 / (self.diffusers.len() as f32) / 4.0;
            let mut reverb_output = [
                (frame8[0] + frame8[2] + frame8[4] + frame8[6]) * scale * reverb_volume,
                (frame8[1] + frame8[3] + frame8[5] + frame8[7]) * scale * reverb_volume,
            ];
            reverb_output[0] = self.filter[0].s_process(reverb_output[0]);
            reverb_output[1] = self.filter[1].s_process(reverb_output[1]);

            frame[0] = reverb_output[0] + left * (1.0 - reverb_volume);
            frame[1] = reverb_output[1] + right * (1.0 - reverb_volume);
        }
    }
}

struct Diffuser {
    rng: SmallRng,
    max_delay_time: Duration,
    #[allow(dead_code)]
    shuffle_positions: [usize; 8],
    mono_delay_processors: [MonoDelayProcessor<f32>; 8],
    delay_times: [f32; 8],
}

impl Default for Diffuser {
    fn default() -> Self {
        let rng = SmallRng::from_entropy();
        Self::new(rng)
    }
}

impl Diffuser {
    fn new(rng: SmallRng) -> Self {
        let shuffle_positions = [2, 3, 1, 0, 5, 4, 7, 6];
        Self {
            rng,
            shuffle_positions,
            max_delay_time: Duration::from_secs_f32(0.0_f32),
            mono_delay_processors: [
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
                MonoDelayProcessor::default(),
            ],
            delay_times: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }

    fn prepare(&mut self, settings: AudioProcessorSettings) {
        let max_delay = self.max_delay_time.as_secs_f32();
        let mut slots: Vec<f32> = (0..self.mono_delay_processors.len())
            .map(|i| (i + 1) as f32 * (max_delay / (self.mono_delay_processors.len() as f32)))
            .collect();

        for (d, delay_time) in self
            .mono_delay_processors
            .iter_mut()
            .zip(&mut self.delay_times)
        {
            d.s_prepare(settings);
            let index = self.rng.gen_range(0..slots.len());
            *delay_time = slots[index];
            slots.remove(index);
            d.handle().set_delay_time_secs(*delay_time);
            d.handle().set_feedback(0.1);
        }
    }

    fn set_delay_mult(&mut self, mult: f32) {
        for (delay, delay_basis) in self.mono_delay_processors.iter_mut().zip(&self.delay_times) {
            delay.handle().set_delay_time_secs(*delay_basis * mult);
        }
    }

    fn process(&mut self, frame: &mut [f32; 8]) {
        for (sample, delay_processor) in frame.iter_mut().zip(&mut self.mono_delay_processors) {
            *sample = delay_processor.s_process(*sample);
        }
        // shuffle(&self.shuffle_positions, frame);
        flip_polarities(frame);
        hadamard_matrix(frame);
    }
}

#[cfg(test)]
mod test {
    use assert_no_alloc::assert_no_alloc;

    use super::*;

    #[test]
    fn test_no_alloc_diffuser() {
        let mut diffuser = Diffuser::default();
        let mut settings = AudioProcessorSettings::default();
        settings.input_channels = 4;
        settings.output_channels = 4;
        diffuser.prepare(settings);

        let mut frame = [0.0, 0.0, 0.0, 0.0];
        assert_no_alloc(|| {
            diffuser.process(&mut frame);
        });
    }
}
