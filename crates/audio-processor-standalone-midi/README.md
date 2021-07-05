# audio-processor-standalone-midi
Wraps `midir` to provide MIDI input handling. The host may be started with `MidiHost`.

When MIDI messages are received, they're pushed onto a lock-free `atomic_queue::Queue`. The messages are picked-up in
the audio-thread by `MidiAudioThreadHandler`.

This crate provides conversion into the VST types, which is to allow a VST host to use it. This is provided by the
`MidiVSTConverter`.

Currently, MIDI messages over 3 bytes are dropped by this host. In addition, the queue is bounded & a size must be
provided. `Default` implementations will use a MIDI queue capacity of 100.