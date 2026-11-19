"""Read-only NumberBox and PersonPicture specifications from local official sources."""

import json
import re

from reference import find_lines, resource_report, revision, source_parser


def main():
    args = source_parser(__doc__).parse_args()
    paths = {
        "controls/dev/NumberBox/NumberBox.idl": r'MUX_DEFAULT_VALUE|ValidationMode|AcceptsExpression',
        "controls/dev/NumberBox/NumberBox.xaml": r'Name="(?:UpDownPopup|PopupContentRoot|UpSpinButton|DownSpinButton|PopupUpSpinButton|PopupDownSpinButton|SpinButtonsColumn|PopupIndicator)"|Property="(?:MinWidth|Width|Height|FontSize|IsTabStop)"',
        "controls/dev/NumberBox/NumberBox.cpp": r'FractionDigits\(0\)|SignificantDigits\(10\)|void NumberBox::(?:ValidateInput|StepValue|CoerceValue|OnNumberBoxGotFocus|OnNumberBoxLostFocus)|VirtualKey::|IsWrapEnabled|std::isnan',
        "controls/dev/NumberBox/NumberBoxParser.cpp": r'c_numberBoxOperators|GetPrecedenceValue|op1 == 0|case L|expectNumber',
        "controls/dev/PersonPicture/PersonPicture.xaml": r'Property="(?:Width|Height|FontWeight|IsTabStop)"|Name="(?:BadgeGrid|InitialsTextBlock|PersonPictureEllipse)"',
        "controls/dev/PersonPicture/PersonPicture.cpp": r'void PersonPicture::(?:UpdateIfReady|UpdateBadge|UpdateAutomationName|OnSizeChanged)|newSize \* 0.5|Width\(\) \* .42|Height\(\) \* 0.6|L"99\+"|if \(IsGroup\(\)\)',
        "controls/dev/PersonPicture/InitialsGenerator.cpp": r'InitialsFromDisplayName|StripTrailingBrackets|i < 3|character [<>]=?|0x[0-9A-F]+|maxIterations',
        "controls/dev/PersonPicture/PersonPictureAutomationPeer.cpp": r'AutomationControlType::Text',
    }
    source = {**revision(args.winui_source), "evidence": {p: find_lines(args.winui_source, p, pattern) for p, pattern in paths.items()}}
    source["resources"] = {name: resource_report(args.winui_source, f"controls/dev/{name}/{name}_themeresources.xaml", re.compile(name + r"\w+")) for name in ["NumberBox", "PersonPicture"]}
    gallery = {**revision(args.gallery_source), "evidence": {f"WinUIGallery/Samples/{name}/{name}Page.xaml": find_lines(args.gallery_source, f"WinUIGallery/Samples/{name}/{name}Page.xaml", name + r'|Value=|Header=|ProfilePicture=|DisplayName=|Initials=') for name in ["NumberBox", "PersonPicture"]}}
    print(json.dumps({"winui": source, "gallery": gallery}, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
