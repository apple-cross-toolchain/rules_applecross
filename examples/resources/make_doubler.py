"""Makes Doubler.mlpackage, the ML Program examples/ios/HelloWorldSwift
compiles and generates a Swift class for: y = 2x for a 1 x 4 array.

    python3 -m venv /tmp/venv && /tmp/venv/bin/pip install coremltools
    /tmp/venv/bin/python examples/resources/make_doubler.py examples/resources/Doubler.mlpackage

Its user-defined metadata is left empty: Xcode writes that map in a randomly
seeded order, so a model with several entries compiles differently from run
to run (docs/clean-room/coremlc.md).
"""

import json
import os
import shutil
import sys

import coremltools as ct
import numpy as np
from coremltools.converters.mil import Builder as mb


@mb.program(input_specs=[mb.TensorSpec(shape=(1, 4))], opset_version=ct.target.iOS17)
def doubler(x):
    return mb.mul(x=x, y=np.float32(2), name="y")


model = ct.convert(doubler, convert_to="mlprogram", minimum_deployment_target=ct.target.iOS17,
                   compute_precision=ct.precision.FLOAT32)
spec = model.get_spec()
spec.description.metadata.shortDescription = "Doubles its input"
spec.description.metadata.userDefined.clear()
ct.models.MLModel(spec, weights_dir=model.weights_dir).save(sys.argv[1])

# The weights are all inline, so the package's weights directory is empty, and
# git keeps no empty directory: drop it from the manifest, or coremlc fails
# to find it.
manifest_path = os.path.join(sys.argv[1], "Manifest.json")
with open(manifest_path) as f:
    manifest = json.load(f)
manifest["itemInfoEntries"] = {k: v for k, v in manifest["itemInfoEntries"].items() if v["name"] != "weights"}
with open(manifest_path, "w") as f:
    f.write(json.dumps(manifest, indent=4) + "\n")
shutil.rmtree(os.path.join(sys.argv[1], "Data", "com.apple.CoreML", "weights"), ignore_errors=True)
