"""Extract WinUI Slider/ProgressBar/ProgressRing resources, behavior and Gallery examples."""

import json
from pathlib import Path
import re

from reference import XAML, find_lines, read_xml, resource_report, revision, source_parser, tag


TEMPLATES = {
    "Slider": Path("controls/dev/CommonStyles/Slider_themeresources.xaml"),
    "ProgressBar": Path("controls/dev/ProgressBar/ProgressBar.xaml"),
    "ProgressRing": Path("controls/dev/ProgressRing/ProgressRing.xaml"),
}
COMMON = Path("controls/dev/CommonStyles/Common_themeresources_any.xaml")


def report_control(winui, gallery, name, template):
    resources = template.with_name(f"{name}_themeresources.xaml")
    samples = gallery / "WinUIGallery" / "Samples" / name
    document = read_xml(winui, template)
    return {
        "template": template.as_posix(),
        "resources": resources.as_posix(),
        "theme_values": resource_report(winui, resources, re.compile(fr"{name}[A-Z].*")),
        "states": [node.get(XAML + "Name") for node in document.iter() if tag(node) == "VisualState"],
        "frame_locations": find_lines(winui, template, r"KeyTime=|Duration="),
        "gallery_samples": {
            path.relative_to(gallery).as_posix(): path.read_text(encoding="utf-8-sig")
            for path in sorted(samples.glob("*.txt"))
        },
    }


def main():
    args = source_parser(__doc__).parse_args()
    winui, gallery = args.winui_source, args.gallery_source
    controls = {name: report_control(winui, gallery, name, template) for name, template in TEMPLATES.items()}
    slider_source = Path("dxaml/xcp/dxaml/lib/Slider_Partial.cpp")
    tick_bar = Path("dxaml/xcp/dxaml/lib/TickBar_Partial.cpp")
    slider_keys = Path("dxaml/xcp/components/controls/KeyDownUp/inc/SliderKeyProcess.h")
    bar_source = TEMPLATES["ProgressBar"].with_suffix(".cpp")
    ring_sources = TEMPLATES["ProgressRing"].parent / "AnimatedVisuals"
    implementation = {
        slider_source: r"GetClosestStep|SliderSnapsTo_|clickPercentage",
        tick_bar: r"MIN_TICKMARK_GAP|numIntervals|singlePixelWidthScaled|tickOffset",
        slider_keys: r"VirtualKey_(Left|Right|Up|Down|Home|End)",
        bar_source: r"maxIndicatorWidth|Container[2]?Animation(Start|End)Position",
        **{
            path.relative_to(winui): r"c_durationTicks|\.(Radius|StrokeThickness|InsertKeyFrame|TransformMatrix)\("
            for path in sorted((winui / ring_sources).glob("ProgressRing*.cpp"))
        },
    }
    print(json.dumps({
        "winui": revision(winui),
        "gallery": revision(gallery),
        "controls": controls,
        "common_resources": resource_report(winui, COMMON, re.compile(
            r"Control(ElevationBorderBrush|Strong(Fill|Stroke)Color(Default|Disabled)|SolidFillColorDefault"
            r"|FastAnimationDuration|NormalAnimationDuration|FastOutSlowInKeySpline)|SystemFillColor(Caution|Critical)"
        )),
        "implementation_locations": {
            path.as_posix(): find_lines(winui, path, pattern) for path, pattern in implementation.items()
        },
    }, indent=2))


if __name__ == "__main__":
    main()
