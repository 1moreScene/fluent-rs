"""Read-only official RepeatButton and HyperlinkButton evidence extraction."""

import json
import re

from reference import find_lines, resource_report, revision, source_parser


def main():
    args = source_parser(__doc__).parse_args()
    source = args.winui_source
    styles = {
        name: f"controls/dev/CommonStyles/{name}_themeresources.xaml"
        for name in ["RepeatButton", "HyperlinkButton"]
    }
    files = {
        **{path: r'Property="(?:Padding|FontSize|CornerRadius|BackgroundSizing)"|Duration="0:0:0.083"' for path in styles.values()},
        "dxaml/xcp/components/DependencyObject/DependencyProperty.cpp": r'RepeatButton_(?:Delay|Interval)|SetSigned\((?:500|33)\)',
        "dxaml/xcp/dxaml/lib/RepeatButton_Partial.cpp": r'ClickMode_Press|newDelay < 0|newInterval <= 0|VirtualKey_Space|m_keyboardCausingRepeat =|m_pointerCausingRepeat =|get_Delay|put_Interval|StartTimer\(\)|StopTimer\(\)',
        "dxaml/xcp/dxaml/lib/HyperlinkButton_Partial.cpp": r'MouseCursorHand|HyperlinkButtonGenerated::OnClick|TryInvokeLauncher|underlineVisibleResourceDirective|bool underline =',
        "dxaml/xcp/components/controls/KeyDownUp/inc/ButtonBaseKeyProcess.h": r'VirtualKey_Enter|VirtualKey_Space|ClickMode_Press|ClickMode_Release|OnClick\(\)',
        "controls/dev/CommonStyles/Hyperlink_themeresources.xaml": r'HyperlinkUnderlineVisible',
        "controls/dev/CommonStyles/Common_themeresources_any.xaml": r'Key="AccentTextFillColor(?:Primary|Secondary|Tertiary|Disabled)(?:Brush)?"',
        "dxaml/xcp/dxaml/themes/generic.xaml": r'Key="SystemControl(?:BackgroundBase(?:Low|MediumLow)|HighlightBase(?:High|MediumLow)|ForegroundTransparent|HighlightTransparent|DisabledTransparent|HyperlinkText|PageTextBaseMedium|PageBackgroundTransparent)Brush"',
        "dxaml/xcp/components/theminginterop/SystemThemingInterop.cpp": r'case VariantAccentColors::|return 0xFF|UIColorType_Accent',
    }
    report = {
        "winui": {**revision(source), "evidence": {p: find_lines(source, p, pattern) for p, pattern in files.items()},
                  "styles": {name: resource_report(source, path, re.compile(name + r"\w+")) for name, path in styles.items()}},
        "gallery": {**revision(args.gallery_source), "evidence": {
            f"WinUIGallery/Samples/{name}/{name}Page.xaml": find_lines(args.gallery_source, f"WinUIGallery/Samples/{name}/{name}Page.xaml", name + r'|NavigateUri=|Content=|IsEnabled=')
            for name in styles
        }},
    }
    print(json.dumps(report, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
