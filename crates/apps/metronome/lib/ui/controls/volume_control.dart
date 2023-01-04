import 'package:flutter/cupertino.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:metronome/modules/state/metronome_state_controller.dart';

class VolumeControl extends StatelessWidget {
  const VolumeControl({
    Key? key,
    required this.stateController,
  }) : super(key: key);

  final MetronomeStateController stateController;

  @override
  Widget build(BuildContext context) {
    final model = stateController.model;
    return Observer(
      builder: (_) => Column(
        children: [
          const Text("Volume"),
          SizedBox(
            width: double.infinity,
            child: CupertinoSlider(
              value: model.volume,
              onChanged: (value) {
                stateController.setVolume(value);
              },
            ),
          ),
        ],
      ),
    );
  }
}
