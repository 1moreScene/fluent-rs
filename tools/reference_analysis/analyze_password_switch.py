"""Extract official PasswordBox and ToggleSwitch styles and input semantics."""

import json
import re

from reference import find_lines, resource_report, revision, source_parser


def collect(root, files):
    return {**revision(root), "evidence": {
        path: find_lines(root, path, pattern) for path, pattern in files.items()
    }}


def main():
    args = source_parser(__doc__).parse_args()
    password = "controls/dev/CommonStyles/PasswordBox_themeresources.xaml"
    switch = "controls/dev/CommonStyles/ToggleSwitch_themeresources.xaml"
    winui = collect(args.winui_source, {
        password: r'PasswordBoxIconFontSize|TextBoxInnerButtonMargin|x:Name="RevealButton"|TextControlTheme(?:Padding|MinHeight)|IsTabStop="False"',
        switch: r'ToggleSwitch(?:ThemeMinWidth|PreContentMargin|PostContentMargin|TopHeaderMargin)|Value="(?:12|14|17)"|RepositionThemeAnimation|x:Name="(?:OuterBorder|SwitchKnob|SwitchThumb)"',
        "controls/dev/CommonStyles/Common_themeresources_any.xaml": r'Key="(?:ControlFasterAnimationDuration|ControlFastOutSlowInKeySpline|ControlAltFillColorDisabled|CircleElevationBorderBrush)"',
        "dxaml/xcp/core/native/text/Controls/PasswordBox.cpp": r'minimumWidthToEnableButton|m_fCanShowRevealButton|AppendChar\(0x25CF\)|PasswordRevealMode::|CanInvokeRevealButton',
        "dxaml/xcp/dxaml/lib/PasswordBox_Partial.cpp": r'VirtualKey_F8|keyStatus.IsMenuKeyDown|newRevealButtonSize.Height|ButtonBase_IsPressed|RevealPassword\(',
        "dxaml/xcp/dxaml/lib/ToggleSwitch_Partial.cpp": r'wasToggled =|m_wasDragged|HandlesKey\(|VirtualKey_Space|add_Tapped|MoveCompleted',
        "dxaml/xcp/dxaml/lib/ThemeAnimations.cpp": r'RepositionThemeAnimation::CreateTimelines|AddTimelinesForThemeAnimation\(TAS_REPOSITION',
    })
    winui["resources"] = {
        path: resource_report(args.winui_source, path, re.compile(prefix + r"\w+"))
        for path, prefix in [(password, "PasswordBox"), (switch, "ToggleSwitch")]
    }
    gallery = collect(args.gallery_source, {
        "WinUIGallery/Samples/PasswordBox/PasswordBoxPage.xaml": r'PasswordBox|PasswordChar|PasswordRevealMode|PlaceholderText',
        "WinUIGallery/Samples/ToggleSwitch/ToggleSwitchPage.xaml": r'ToggleSwitch|Header=|OffContent=|OnContent=|IsOn=',
    })
    print(json.dumps({"winui": winui, "gallery": gallery}, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
