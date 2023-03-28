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

use std::slice::{Chunks, ChunksMut};

/// Represents an audio buffer. This decouples audio processing code from a certain representation
/// of multi-channel sample buffers.
///
/// This crate provides implementations of this trait for CPal style buffers, which use interleaved
/// internal representation.
///
/// When processing samples, it'll be more efficient to use `.slice` and `.slice_mut` than `.get` /
/// `.set` methods. For the VST buffer, these methods will not work.
///
/// It's recommended to convert the buffer into interleaved layout before processing as that will be
/// around as expensive as the overhead of `get`/`set` methods on a single loop through samples.
///
/// (due to bounds checking and other compiler optimisations that fail with them)
pub trait AudioBuffer {
    /// The type of samples within this buffer.
    type SampleType;

    /// The number of channels in this buffer
    fn num_channels(&self) -> usize;

    /// The number of samples in this buffer
    fn num_samples(&self) -> usize;

    /// Get a slice to the internal data. Will not work with VST adapter
    ///
    /// This is the faster way to process
    fn slice(&self) -> &[Self::SampleType];

    /// Get a mutable slice to the internal data. Will not work with VST adapter
    ///
    /// This is the faster way to process
    fn slice_mut(&mut self) -> &mut [Self::SampleType];

    /// Shortcut for `.slice().chunks(num_channels)`
    fn frames(&self) -> Chunks<'_, Self::SampleType> {
        self.slice().chunks(self.num_channels())
    }

    /// Shortcut for `.slice_mut().chunks_mut(num_channels)`
    ///
    /// This is a frame representing a sample in time, for all
    /// channels.
    fn frames_mut(&mut self) -> ChunksMut<'_, Self::SampleType> {
        let channels = self.num_channels();
        self.slice_mut().chunks_mut(channels)
    }

    /// Get a ref to an INPUT sample in this buffer.
    ///
    /// Calling this on a loop will be ~20x slower than reading from `slice`.
    fn get(&self, channel: usize, sample: usize) -> &Self::SampleType;

    /// Get a mutable ref to an OUTPUT sample in this buffer
    ///
    /// On some implementations this may yield a different value than `.get`.
    ///
    /// Calling this on a loop will be ~20x slower than reading from `slice`.
    fn get_mut(&mut self, channel: usize, sample: usize) -> &mut Self::SampleType;

    /// Set an OUTPUT sample in this buffer
    fn set(&mut self, channel: usize, sample: usize, value: Self::SampleType);

    /// Unsafe, no bounds check - Get a ref to an INPUT sample in this buffer
    ///
    /// Calling this on a loop will be ~10x slower than reading from `slice`.
    ///
    /// # Safety
    /// This performs no bounds checks. Make sure indexes are in range.
    unsafe fn get_unchecked(&self, channel: usize, sample: usize) -> &Self::SampleType {
        self.get(channel, sample)
    }

    /// Unsafe, no bounds check - Get a mutable ref to an OUTPUT sample in this buffer
    ///
    /// On some implementations this may yield a different value than `.get`.
    ///
    /// Calling this on a loop will be ~10x slower than reading from `slice`.
    ///
    /// # Safety
    /// This performs no bounds checks. Make sure indexes are in range.
    unsafe fn get_unchecked_mut(&mut self, channel: usize, sample: usize) -> &mut Self::SampleType {
        self.get_mut(channel, sample)
    }

    /// Unsafe, no bounds check - Set an OUTPUT sample in this buffer
    ///
    /// Calling this on a loop will be ~10x slower than reading from `slice`.
    ///
    /// # Safety
    /// This performs no bounds checks. Make sure indexes are in range.
    unsafe fn set_unchecked(&mut self, channel: usize, sample: usize, value: Self::SampleType) {
        self.set(channel, sample, value)
    }
}
