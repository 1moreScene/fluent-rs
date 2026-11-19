"""Read AutoSuggestBox resources, input semantics, and official Gallery examples."""

import json
import re

from reference import find_lines, resource_report, revision, source_parser


def report(root, files):
    return {
        **revision(root),
        "evidence": {path: find_lines(root, path, pattern) for path, pattern in files.items()},
    }


def main():
    args = source_parser(__doc__).parse_args()
    template = "controls/dev/AutoSuggestBox/AutoSuggestBox_themeresources.xaml"
    winui = report(args.winui_source, {
        template: r"AutoSuggestBox(?:InnerButtonMargin|DeleteButtonMargin|RightButtonMargin|IconFontSize)|QueryButton\"|KeepInteriorCornersSquare|SuggestionsContainer|SuggestionsList\"",
        "dxaml/xcp/dxaml/themes/generic.xaml": r'x:Key="AutoSuggestList(?:MaxHeight|Margin|Padding|BorderThemeThickness)"',
        "dxaml/xcp/dxaml/lib/AutoSuggestBox_Partial.cpp": r"AutoSuggestBox::(?:OnKeyDown|OnTextBoxTextChanged|SubmitQuery|ProgrammaticSubmitQuery|OnSuggestionSelectionChanged|MaximizeSuggestionAreaWithoutInputPane)|VirtualKey_(?:Tab|Escape|Enter)|m_userTypedText|deltaTop < deltaBottom|AutoSuggestionBoxTextChangeReason_\w+",
        "controls/dev/AutoSuggestBox/AutoSuggestBoxHelper.cpp": r"UpdateCornerRadius|CornerRadiusFilterKind::|CornerRadius\(",
        "controls/dev/CommonStyles/ListViewItem_themeresources.xaml": r'Key="ListViewItem(?:MinHeight|CornerRadius|SelectionIndicatorCornerRadius)"|Property="Padding"',
        "dxaml/xcp/core/core/elements/ListViewBaseItemChrome.cpp": r"s_selectionIndicator(?:Size|Margin|HeightShrinkage)|s_backplateMargin",
    })
    winui["resources"] = resource_report(args.winui_source, template, re.compile(r"AutoSuggestBox\w+"))
    gallery = report(args.gallery_source, {
        "WinUIGallery/Samples/AutoSuggestBox/AutoSuggestBoxPage.xaml": r"AutoSuggestBox|QueryIcon|SuggestionChosen|TextChanged|QuerySubmitted|Width=",
        "WinUIGallery/Samples/AutoSuggestBox/AutoSuggestBoxPage.xaml.cs": r"AutoSuggestionBoxTextChangeReason.UserInput|ChosenSuggestion|SuggestionChosen|QuerySubmitted|SearchControls|Contains\(key\)",
    })
    print(json.dumps({"winui": winui, "gallery": gallery}, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
