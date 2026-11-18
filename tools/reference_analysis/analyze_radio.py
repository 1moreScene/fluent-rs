"""Extract the WinUI RadioButton template and RadioButtons keyboard evidence."""

import json
from pathlib import Path
import re

from reference import XAML, find_lines, read_xml, resource_report, revision, source_parser, tag


def main():
    args = source_parser(__doc__).parse_args()
    template = Path("controls/dev/CommonStyles/RadioButton_themeresources.xaml")
    group_resources = Path("controls/dev/RadioButtons/RadioButtons_themeresources.xaml")
    style = next(
        node for node in read_xml(args.winui_source, template)
        if node.get(XAML + "Key") == "DefaultRadioButtonStyle"
    )
    page = Path("WinUIGallery/Samples/RadioButton/RadioButtonPage.xaml")
    input_sources = {
        "dxaml/xcp/dxaml/lib/RadioButton_Partial.cpp":
            r"SetAcceptsReturn|RadioButton::OnToggle|VisualRelativeKind_Root|VirtualKey_(Left|Right|Up|Down)|we loop around",
        "controls/dev/RadioButtons/RadioButtons.cpp":
            r"RadioButtons::OnGettingFocus|Selection follows focus|VirtualKey::(Control|Left|Right|Up|Down)\b|RadioButtons::MoveFocus\(",
        "controls/dev/RadioButtons/InteractionTests/RadioButtonsTests.cs":
            r"BasicKeyboardTest|MultiColumnKeyboardTest|DisabledItemsKeyboardTest|ControlKeyKeyboardTest",
    }
    report = {
        "winui": revision(args.winui_source),
        "gallery": revision(args.gallery_source),
        "template": template.as_posix(),
        "setters": {node.get("Property"): node.get("Value") for node in style if tag(node) == "Setter"},
        "states": [node.get(XAML + "Name") for node in style.iter() if tag(node) == "VisualState"],
        "radio_tokens": resource_report(args.winui_source, template, re.compile(r"RadioButton.*")),
        "group_tokens": resource_report(args.winui_source, group_resources, re.compile(r"RadioButtons.*")),
        "input": {
            name: find_lines(args.winui_source, Path(name), pattern)
            for name, pattern in input_sources.items()
        },
        "gallery_page": {
            "path": page.as_posix(),
            "locations": find_lines(args.gallery_source, page, r"RadioButtons|MaxColumns=|SelectedIndex=|SampleDefinition="),
        },
    }
    print(json.dumps(report, ensure_ascii=True, indent=2))


if __name__ == "__main__":
    main()
