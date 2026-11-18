"""Read the Microsoft Button templates and Gallery samples into a JSON report."""

import json
from pathlib import Path
import re

from reference import XAML, find_lines, read_xml, resource_report, revision, source_parser, tag

BUTTON_TEMPLATE = Path("controls/dev/CommonStyles/Button_themeresources.xaml")
GALLERY_PAGE = Path("WinUIGallery/Samples/Button/ButtonPage.xaml")
BUTTON_BASE = Path("dxaml/xcp/dxaml/lib/ButtonBase_Partial.cpp")
BUTTON = Path("dxaml/xcp/dxaml/lib/Button_Partial.cpp")
STYLES = {"DefaultButtonStyle", "AccentButtonStyle", "SubtleButtonStyle"}
TOKEN_FILES = (
    BUTTON_TEMPLATE,
    Path("controls/dev/CommonStyles/Common_themeresources_any.xaml"),
    Path("controls/dev/CommonStyles/CornerRadius_themeresources.xaml"),
    Path("dxaml/xcp/dxaml/themes/generic.xaml"),
)
TOKEN_KEYS = re.compile(
    r"ButtonPadding|ControlCornerRadius|ControlContentThemeFontSize|ContentControlThemeFontFamily|"
    r"SolidBackgroundFillColorBase|Text(?:OnAccent)?FillColor(?:Primary|Secondary|Disabled)|"
    r"(?:Control|Subtle|Accent)FillColor(?:Default|Secondary|Tertiary|Disabled|Transparent)(?:Brush)?|"
    r"ControlStrokeColor(?:OnAccent)?(?:Default|Secondary)|FocusStrokeColor(?:Inner|Outer)|"
    r"(?:Accent)?ControlElevationBorderBrush"
)


def template_report(template):
    styles = {}
    for style in template:
        key = style.get(XAML + "Key")
        if key not in STYLES:
            continue
        styles[key] = {
            "setters": {s.get("Property"): s.get("Value") for s in style if tag(s) == "Setter"},
            "transitions": [node.attrib for node in style.iter() if tag(node) == "BrushTransition"],
            "states": {
                node.get(XAML + "Name"): [
                    keyframe.get("Value") for keyframe in node.iter()
                    if tag(keyframe) == "DiscreteObjectKeyFrame"
                ]
                for node in style.iter() if tag(node) == "VisualState"
            },
        }
    dictionaries = {
        dictionary.get(XAML + "Key"): {
            node.get(XAML + "Key"): node.get("ResourceKey") or (node.text or "").strip()
            for node in dictionary
            if tag(node) in {"StaticResource", "Thickness"}
        }
        for child in template if tag(child) == "ResourceDictionary.ThemeDictionaries"
        for dictionary in child
    }
    return styles, dictionaries


def main():
    parser = source_parser(__doc__)
    args = parser.parse_args()

    styles, dictionaries = template_report(read_xml(args.winui_source, BUTTON_TEMPLATE))
    if set(styles) != STYLES:
        parser.error(f"Button styles changed in {BUTTON_TEMPLATE}: {', '.join(styles)}")
    page = read_xml(args.gallery_source, GALLERY_PAGE)
    report = {
        "winui": revision(args.winui_source),
        "gallery": revision(args.gallery_source),
        "template": {
            "path": BUTTON_TEMPLATE.as_posix(),
            "locations": find_lines(args.winui_source, BUTTON_TEMPLATE, r'ButtonPadding|Duration="[^"]+"|x:Name="(?:Normal|PointerOver|Pressed|Disabled)"|x:Key="\w+ButtonStyle"'),
            "styles": styles,
            "themes": dictionaries,
        },
        "tokens": {path.as_posix(): resource_report(args.winui_source, path, TOKEN_KEYS) for path in TOKEN_FILES},
        "input": {
            path.as_posix(): find_lines(args.winui_source, path, r'ClickMode_Release|FocusState_(?:Pointer|Keyboard)|OnKeyDownInternal|OnKeyUpInternal|OnPointer(?:Pressed|Released)')
            for path in (BUTTON_BASE, BUTTON)
        },
        "gallery_page": {
            "path": GALLERY_PAGE.as_posix(),
            "buttons": [node.attrib for node in page.iter() if tag(node) == "Button"],
            "locations": find_lines(args.gallery_source, GALLERY_PAGE, r'Click="Button_Click"|IsEnabled="[^\"]+"|(?:Accent|Subtle)ButtonStyle|ButtonWrapping\.txt'),
            "click_handler": find_lines(args.gallery_source, GALLERY_PAGE.with_suffix(".xaml.cs"), r'Button_Click|Control1Output\.Text|Control2Output\.Text'),
        },
    }
    print(json.dumps(report, ensure_ascii=True, indent=2))


if __name__ == "__main__":
    main()
