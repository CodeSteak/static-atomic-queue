//
//  File.swift
//
//
//  Created by Pedro Tacla Yamada on 19/3/2022.
//

import SwiftUI

struct TopBarView: View {
    @EnvironmentObject var store: Store
    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 0) {
                TransportTempoView(timeInfo: store.timeInfo).frame(width: 80, alignment: .trailing)
                Spacer()
                TransportControlsView()

                Spacer()

                Button(
                    "MIDI",
                    action: {}
                )
                .padding(PADDING * 0.5)
                .background(SequencerColors.black3)
                .buttonStyle(.plain)
            }
            .padding(PADDING)

            Rectangle()
                .fill(SequencerColors.white.opacity(0.1))
                .frame(height: 1.0).frame(maxWidth: .infinity)
        }
        .frame(maxWidth: .infinity)
        .background(SequencerColors.black)
    }
}
