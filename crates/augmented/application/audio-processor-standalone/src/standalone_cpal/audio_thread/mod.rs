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

use std::sync::mpsc::{Receiver, Sender};

use cpal::traits::StreamTrait;
use cpal::{Device, Stream, StreamConfig};

use audio_processor_traits::{AudioProcessor, AudioProcessorSettings};

use crate::StandaloneProcessor;

use super::{
    error::AudioThreadError, input_handling, midi::MidiContext, options, output_handling,
    IOConfiguration, ResolvedStandaloneConfiguration,
};

pub fn audio_thread_main<SP: StandaloneProcessor>(
    mut app: SP,
    midi_context: Option<MidiContext>,
    configuration_tx: Sender<ResolvedStandaloneConfiguration>,
    stop_signal_rx: Receiver<()>,
) {
    // Audio set-up
    let host = cpal::default_host();
    let host_name = host.id().name().to_string();
    log::info!("Using host: {}", host.id().name());
    let buffer_size = 512;
    let sample_rate = {
        #[cfg(not(target_os = "ios"))]
        {
            44100
        }
        #[cfg(target_os = "ios")]
        {
            48000
        }
    };

    let options = app.options();
    let accepts_input = options.accepts_input;
    let input_tuple = if accepts_input {
        Some(options::configure_input_device(
            &host,
            &options,
            buffer_size,
            sample_rate,
        ))
    } else {
        None
    };
    let (output_device, output_config) =
        options::configure_output_device(host, &options, buffer_size, sample_rate);

    let num_output_channels = output_config.channels.into();
    let num_input_channels = input_tuple
        .as_ref()
        .map(|(_, input_config)| input_config.channels.into())
        .unwrap_or(num_output_channels);

    let settings = AudioProcessorSettings::new(
        output_config.sample_rate.0 as f32,
        num_input_channels,
        num_output_channels,
        buffer_size,
    );
    app.processor().prepare(settings);

    configuration_tx
        .send(ResolvedStandaloneConfiguration {
            host: host_name,
            input_configuration: input_tuple
                .as_ref()
                .map(|(input_device, config)| IOConfiguration::new(input_device, config)),
            output_configuration: IOConfiguration::new(&output_device, &output_config),
        })
        .unwrap();

    let run_params = AudioThreadRunParams {
        io_hints: AudioThreadIOHints {
            buffer_size,
            num_output_channels,
            num_input_channels,
        },
        cpal_streams: AudioThreadCPalStreams {
            output_config,
            input_tuple,
            output_device,
        },
        midi_context,
    };

    let streams = audio_thread_run_processor(run_params, app);

    let _ = stop_signal_rx.recv();

    drop(streams);
}

/// At this point we have "negotiated" the nº of channels and buffer size.
/// These will be used in logic on the callbacks as well as to size our ringbuffer.
struct AudioThreadIOHints {
    buffer_size: usize,
    num_output_channels: usize,
    num_input_channels: usize,
}

/// Input and output audio streams.
struct AudioThreadCPalStreams {
    output_config: StreamConfig,
    input_tuple: Option<(Device, StreamConfig)>,
    output_device: Device,
}

struct AudioThreadRunParams {
    midi_context: Option<MidiContext>,
    io_hints: AudioThreadIOHints,
    cpal_streams: AudioThreadCPalStreams,
}

/// Start this processor with given run parameters.
/// The processor should be prepared at this point.
fn audio_thread_run_processor(
    params: AudioThreadRunParams,
    app: impl StandaloneProcessor,
) -> Option<(Option<Stream>, Stream)> {
    let AudioThreadRunParams {
        midi_context,
        io_hints,
        cpal_streams,
    } = params;
    let AudioThreadIOHints {
        buffer_size,
        num_output_channels,
        num_input_channels,
    } = io_hints;
    let AudioThreadCPalStreams {
        output_config,
        input_tuple,
        output_device,
    } = cpal_streams;

    let build_streams = move || -> Result<(Option<Stream>, Stream), AudioThreadError> {
        let buffer = ringbuf::RingBuffer::new((buffer_size * 10) as usize);
        let (producer, consumer) = buffer.split();
        let input_stream = input_tuple
            .map(|(input_device, input_config)| {
                input_handling::build_input_stream(input_device, input_config, producer)
            })
            // "invert" Option<Result<...>> to Result<Option<...>, ...>
            .map_or(Ok(None), |v| v.map(|v| Some(v)))?;
        let output_stream = output_handling::build_output_stream(
            app,
            midi_context,
            num_output_channels,
            num_input_channels,
            consumer,
            output_device,
            output_config,
        )?;

        Ok((input_stream, output_stream))
    };

    match build_streams() {
        Ok((input_stream, output_stream)) => {
            log::info!("Audio streams starting on audio-thread");
            let play = || -> Result<(Option<Stream>, Stream), AudioThreadError> {
                if let Some(input_stream) = &input_stream {
                    input_stream
                        .play()
                        .map_err(AudioThreadError::InputStreamError)?;
                }

                output_stream
                    .play()
                    .map_err(AudioThreadError::OutputStreamError)?;

                Ok((input_stream, output_stream))
            };

            match play() {
                Ok(streams) => {
                    log::info!("Audio streams started");
                    Some(streams)
                }
                Err(err) => {
                    log::error!("Audio-thread failed to start with {}", err);
                    None
                }
            }
        }
        Err(err) => {
            log::error!("Audio-thread failed to start with {}", err);
            None
        }
    }
}