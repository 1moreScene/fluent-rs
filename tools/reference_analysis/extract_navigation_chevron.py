"""Extract only vector positions, keyframes and markers from a local WinUI checkout."""
from argparse import ArgumentParser
from pathlib import Path
import re

parser = ArgumentParser(description=__doc__)
parser.add_argument("winui", type=Path)
parser.add_argument("--output", type=Path)
args = parser.parse_args()
source = args.winui / "controls/dev/AnimatedIcon/AnimatedVisuals/AnimatedChevronUpDownSmallVisualSource.cpp"
data = source.read_text()
number = r"(-?[0-9.]+)F"
duration = int(re.search(r"c_durationTicks\{ (\d+)L", data)[1])
names = re.findall(r'\{ L"(\w+_(?:Start|End))", ([\d.]+) \}', data)
rotation = data.split("ScalarKeyFrameAnimation RotationAngleInDegreesScalarAnimation_180_to_180()")[1].split("return result;")[0]
offset = data.split("Vector2KeyFrameAnimation OffsetVector2Animation()")[1].split("return result;")[0]

def easing(name):
    digit = re.search(r"Function_(\d)", name)
    return digit[1] if digit else "4"

def track(text, expression):
    return [(*row[:-1], easing(row[-1])) for row in re.findall(expression, text)]

rotations = track(rotation, r"InsertKeyFrame\(" + number + ", " + number + r", (.*)\);")
offsets = track(offset, r"InsertKeyFrame\(" + number + ", \{ " + number + ", " + number + r" \}, (.*)\);")
curves = [row for row in re.findall(r"CreateCubicBezierEasingFunction\(\{ " + number + ", " + number + r" \}, \{ " + number + ", " + number + r" \}\)", data)]
assert len(names) == 36 and len(curves) == 4 and len(rotations) == 33 and len(offsets) >= 30, (len(names),len(curves),len(rotations),len(offsets))
lines = ["// Numeric motion data extracted from WinUI's MIT-licensed AnimatedChevronUpDownSmallVisualSource.",
         "// Copyright (c) Microsoft Corporation. See ui/icons/winui-chevron/LICENSE.",
         "// Regenerate with tools/reference_analysis/extract_navigation_chevron.py.",
         f"pub(super) const DURATION_MS: f32 = {duration / 10_000:.4f};",
         "pub(super) const MARKERS: [(&str, f64); 36] = ["]
lines += [f'    ("{name}", {float(value):.9f}),' for name, value in names]
lines += ["];","pub(super) const CURVES: [[f64; 4]; 4] = ["]
lines += ["    [" + ", ".join(f"{float(v):.3f}" for v in row) + "]," for row in curves]
lines += ["];","pub(super) const ROTATION: &[(f64, [f64; 2], usize)] = &[","    (0.0, [180.0, 0.0], 4),"]
lines += [f"    ({float(frame):.9f}, [{float(value):.1f}, 0.0], {curve})," for frame, value, curve in rotations]
lines += ["];","pub(super) const OFFSET: &[(f64, [f64; 2], usize)] = &[","    (0.0, [24.0, 24.0], 4),"]
lines += [f"    ({float(frame):.9f}, [{float(x):.6f}, {float(y):.6f}], {curve})," for frame, x, y, curve in offsets]
lines += ["];"]
output = "\n".join(lines) + "\n"
if args.output:
    args.output.write_text(output)
else:
    print(output, end="")
