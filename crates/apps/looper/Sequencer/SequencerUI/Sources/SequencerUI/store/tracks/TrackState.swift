// = copyright ====================================================================
// Continuous: Live-looper and performance sampler
// Copyright (C) 2022  Pedro Tacla Yamada
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
// = /copyright ===================================================================
import Combine

public class TrackState: ObservableObject {
    @Published public var id: Int
    @Published var steps: [SequencerStepState?] = (0 ... 16).map { _ in nil }
    @Published var buffer: TrackBuffer? = nil
    @Published public var sliceBuffer: SliceBuffer? = nil // public for ref checks
    @Published public var sourceParameters: SourceParametersState
    @Published public var envelope: EnvelopeState
    @Published public var quantizationParameters: QuantizationParameters

    @Published var volumeParameter: FloatParameter<Int>

    @Published var lfo1: LFOState
    @Published var lfo2: LFOState

    @Published public var looperState: LooperState = .empty

    @Published public var numSamples: UInt = 0

    let position: LoopPosition = .init()
    public var positionPercent: Float {
        get { position.positionPercent }
        set {
            position.positionPercent = newValue
        }
    }

    init(id: Int) {
        self.id = id
        volumeParameter = .init(
            id: 0,
            globalId: .trackVolume(trackId: id),
            label: "Volume \(id)",
            initialValue: 1.0
        )
        sourceParameters = .init(trackId: id)
        quantizationParameters = .init(trackId: id)
        envelope = .init(trackId: id)
        lfo1 = .init(trackId: id, index: 0)
        lfo2 = .init(trackId: id, index: 1)
    }
}

extension TrackState {
    func toggleStep(_ step: Int) {
        if steps[step] != nil {
            steps[step] = nil
        } else {
            steps[step] = SequencerStepState(index: step)
        }
        objectWillChange.send()
    }
}