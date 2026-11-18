"""Extract the official WinUI TextBox template, theme tokens and Gallery samples."""

import json
from pathlib import Path
import re

from reference import XAML, find_lines, read_xml, resource_report, revision, source_parser, tag

COMMON = Path("controls/dev/CommonStyles")
TEMPLATE = COMMON / "TextBox_themeresources.xaml"
NATIVE = Path("dxaml/xcp/core/native/text/Controls")
GALLERY_PAGE = Path("WinUIGallery/Samples/TextBox/TextBoxPage.xaml")
TOKEN_FILES = (
    TEMPLATE, COMMON / "Common_themeresources.xaml", COMMON / "Common_themeresources_any.xaml",
    COMMON / "CornerRadius_themeresources.xaml", Path("dxaml/xcp/dxaml/themes/generic.xaml"),
)
TOKEN_KEYS = re.compile(
    r"TextControl(?:Background|Border|Foreground|Placeholder|Selection|Theme|Header|Elevation|Button)\w*|"
    r"TextBox(?:TopHeaderMargin|InnerButtonMargin|IconFontSize)|TemporaryTextFillColorDisabled|ControlCornerRadius|"
    r"ContentControlThemeFontFamily|ControlContentThemeFontSize|"
    r"ControlFillColor(?:Default|Secondary|InputActive|Disabled)|Control(?:Strong)?StrokeColorDefault|"
    r"TextFillColor(?:Primary|Secondary|Tertiary|Disabled)|AccentFillColorSelectedTextBackgroundBrush"
)


def main():
    parser = source_parser(__doc__)
    args = parser.parse_args()
    source = read_xml(args.winui_source, TEMPLATE)
    style = next((node for node in source if node.get(XAML + "Key") == "DefaultTextBoxStyle"), None)
    if style is None:
        parser.error(f"DefaultTextBoxStyle missing from {TEMPLATE}")
    page = read_xml(args.gallery_source, GALLERY_PAGE)
    groups = {}
    for group in style.iter():
        if tag(group) != "VisualStateGroup":
            continue
        name = group.get(XAML + "Name")
        if name == "CommonStates" and any(tag(node) == "VisualState" and node.get(XAML + "Name") == "Pressed" for node in group):
            name = "DeleteButton.CommonStates"
        groups[name] = {
            state.get(XAML + "Name"): [
                keyframe.get("Value") for keyframe in state.iter()
                if tag(keyframe) == "DiscreteObjectKeyFrame"
            ]
            for state in group if tag(state) == "VisualState"
        }

    report = {
        "winui": revision(args.winui_source),
        "gallery": revision(args.gallery_source),
        "template": {
            "path": TEMPLATE.as_posix(),
            "setters": {node.get("Property"): node.get("Value") for node in style if tag(node) == "Setter"},
            "states": groups,
            "clear_button": next(
                (node.attrib for node in style.iter() if node.get(XAML + "Name") == "DeleteButton"),
                None,
            ),
            "locations": find_lines(args.winui_source, TEMPLATE, r'DefaultTextBoxStyle|x:Name="(?:BorderElement|PlaceholderTextContentPresenter|DeleteButton|Focused)"|TextControlBorderThemeThicknessFocused'),
        },
        "tokens": {path.as_posix(): resource_report(args.winui_source, path, TOKEN_KEYS) for path in TOKEN_FILES},
        "input": {
            path.as_posix(): find_lines(args.winui_source, path, r'IsReadOnly\(\)|OnIsReadOnlyChanged|CanInvokeDeleteButton|GoToState[^;]+')
            for path in (NATIVE / "TextBox.cpp", NATIVE / "TextBoxBase.cpp")
        },
        "clear_button_input": {
            path.as_posix(): find_lines(args.winui_source, path, pattern)
            for path, pattern in (
                (NATIVE / "TextBox.cpp", r'Minimum width for TextBox with DeleteButton visible is 5em|minimumWidthToEnableButton = [^;]+|CTextBox::(?:OnDeleteButtonClick|CanInvokeDeleteButton)|SetTextServicesBuffer\(xstring_ptr::NullString\(\)\)|m_fHasSpaceForDeleteButton &&'),
                (NATIVE / "TextBoxBase.cpp", r'CTextBoxBase::SetTextServicesBuffer|TxSetText\(pNullTermintatedText\)'),
                (Path("dxaml/xcp/dxaml/lib/TextBox_Partial.cpp"), r'UIA_TEXTBOX_DELETE|maintain square button'),
            )
        },
        "gallery_page": {
            "path": GALLERY_PAGE.as_posix(),
            "textboxes": [node.attrib for node in page.iter() if tag(node) == "TextBox"],
            "locations": find_lines(args.gallery_source, GALLERY_PAGE, r'<TextBox|\b(?:IsReadOnly|Header|PlaceholderText|AcceptsReturn|TextWrapping)="[^"]+"'),
        },
    }
    print(json.dumps(report, ensure_ascii=True, indent=2))


if __name__ == "__main__":
    main()
