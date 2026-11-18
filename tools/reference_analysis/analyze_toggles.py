"""Extract WinUI ToggleButton and CheckBox templates and Gallery samples as JSON."""

import json
from pathlib import Path
import re

from reference import XAML, find_lines, read_xml, resource_report, revision, source_parser, tag


def report_control(winui, gallery, name):
    template = Path(f"controls/dev/CommonStyles/{name}_themeresources.xaml")
    style_name = f"Default{name}Style"
    style = next(node for node in read_xml(winui, template) if node.get(XAML + "Key") == style_name)
    page = Path(f"WinUIGallery/Samples/{name}/{name}Page.xaml")
    implementation = Path(f"dxaml/xcp/dxaml/lib/{name}_Partial.cpp")
    return {
        "template": template.as_posix(),
        "setters": {node.get("Property"): node.get("Value") for node in style if tag(node) == "Setter"},
        "transitions": [node.attrib for node in style.iter() if tag(node) == "BrushTransition"],
        "states": [node.get(XAML + "Name") for node in style.iter() if tag(node) == "VisualState"],
        "theme_resources": resource_report(
            winui, template, re.compile(fr"{name}(?:[A-Z].*|Size|Height|Padding)")
        ),
        "input": find_lines(
            winui, implementation, r"SetAcceptsReturn|OnToggleProtected|GoToState|IsThreeState|VirtualKey_\w+"
        ),
        "gallery": {
            "path": page.as_posix(),
            "controls": [node.attrib for node in read_xml(gallery, page).iter() if tag(node) == name],
            "locations": find_lines(gallery, page, r"SampleDefinition=|IsThreeState=|IsChecked="),
        },
    }


def main():
    args = source_parser(__doc__).parse_args()
    report = {
        "winui": revision(args.winui_source),
        "gallery": revision(args.gallery_source),
        "controls": {
            name: report_control(args.winui_source, args.gallery_source, name)
            for name in ("ToggleButton", "CheckBox")
        },
    }
    print(json.dumps(report, ensure_ascii=True, indent=2))


if __name__ == "__main__":
    main()
