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
import SwiftUI

struct TrackButton: View {
    var action: () -> Void
    var label: String
    var isDisabled: Bool = false
    var isSelected: Bool
    var backgroundColor: Color?

    var body: some View {
        Button(
            action: action,
            label: { Text(label)
                .frame(width: 80.0, height: 80.0, alignment: .center)
                .contentShape(Rectangle())
                .foregroundColor(SequencerColors.white)
                .background(
                    RoundedRectangle(cornerRadius: BORDER_RADIUS)
                        .stroke(
                            isSelected ? SequencerColors.red : SequencerColors.black3,
                            lineWidth: 1.0
                        )
                        .background(self.backgroundColor ?? SequencerColors.black)
                )
                .cornerRadius(BORDER_RADIUS)
            }
        )
        .buttonStyle(.plain)
        .disabled(isDisabled)
    }
}