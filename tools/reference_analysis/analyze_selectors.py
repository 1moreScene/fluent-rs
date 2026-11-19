"""Extract ListBox and ComboBox behavior, templates, and official Gallery examples."""

import json
import re

from reference import find_lines, resource_report, source_parser
from analyze_composition import report

LISTBOX = "controls/dev/CommonStyles/ListBox_themeresources.xaml"
COMBOBOX = "controls/dev/ComboBox/ComboBox_themeresources.xaml"
WINUI = {
    LISTBOX: r"ListBoxItemPadding|ListBoxItemBackground(?:Selected|Pressed|PointerOver)\w*|ListBoxBorderThemeThickness|VirtualizingStackPanel|VerticalScrollBarVisibility",
    "dxaml/xcp/dxaml/themes/generic.xaml":
        r'SystemChromeMediumLowColor|x:Key="SystemControlHighlightListAccent(?:Low|Medium|High)Brush"',
    "controls/dev/CommonStyles/Common_themeresources_any.xaml":
        r'x:Key="(?:SubtleFillColor(?:Secondary|Tertiary)|SurfaceStrokeColorFlyout)"',
    "controls/dev/Materials/Acrylic/AcrylicBrush_themeresources.xaml":
        r'<AcrylicBrush x:Key="AcrylicInAppFillColorDefaultBrush"[^>]*>',
    "controls/dev/AnimatedIcon/AnimatedVisuals/AnimatedChevronDownSmallVisualSource.h":
        r"NormalToPressed_End|PressedToNormal_(?:Start|End)",
    "controls/dev/AnimatedIcon/AnimatedVisuals/AnimatedChevronDownSmallVisualSource.cpp":
        r"0\.310344815F|0\.517241359F|31\.5F",
    "controls/dev/ComboBox/ComboBoxHelper.cpp":
        r"UpdateCornerRadius|IsPopupOpenDown|RadiusFilter|\.CornerRadius\(",
    "controls/dev/CommonStyles/ScrollBar_themeresources.xaml":
        r'Key="ScrollBar(?:Size|CornerRadius|ThumbFill|VerticalThumbMin|Expand|Contract|OpacityChange)|x:Name="VerticalPanningThumb"',
    "dxaml/xcp/dxaml/lib/SplitOpenThemeAnimation_Partial.h": r"s_OpacityChangeDuration|s_OpenDuration",
    "dxaml/xcp/dxaml/lib/SplitCloseThemeAnimation_Partial.h": r"s_CloseDuration|s_OpacityChangeDuration|s_OpacityChangeBeginTime",
    "dxaml/xcp/dxaml/lib/ThemeAnimations.cpp":
        r"Split(?:Open|Close)ThemeAnimation::CreateTimelines|const DOUBLE closedRatio|splitOrigin =|finalClipScaleY = \(0\.5|Split(?:Open|Close)ThemeAnimation::s_",
    COMBOBOX: r'ComboBox(?:ThemeMinWidth|MinHeight|PopupMaxNumberOfItems|EditableTextPadding)|x:Name="(?:PointerOver|Pressed|Focused|Disabled|Popup|DropDownGlyph|EditableText|DropDownOverlay)"|Split(?:Open|Close)ThemeAnimation|ComboBoxDropdownContentMargin',
    "dxaml/xcp/core/core/elements/DefaultValues.cpp": r"m_singleSelectionFollowsFocus = true",
    "dxaml/xcp/dxaml/lib/ListBox_Partial.cpp": r"SingleSelectionFollowsFocus|Make(?:Single|Range|Toggle)Selection|SelectAllItems|VirtualKey_(?:Space|Enter|PageUp|PageDown)",
    "dxaml/xcp/dxaml/lib/ComboBox_Partial.cpp":
        r"SelectionChangedTrigger_Always|m_isClosingDueToCancel|VirtualKey_(?:Escape|Tab|Space|Enter|F4)|OverrideSelectedIndexForVisualStates|Get(?:NonPannable|EditableComboBox)PopupLayout|centerItemIndex = itemCount/2|Layout centerItem on faceplate|put_DropDownOffset|PointerFocused|CommitRevertEditableSearch|RaiseTextSubmittedEvent|StartsWithIgnoreLinguisticSemantics|VK_BACK|m_candidateWindowBoundsRect.Height",
    "dxaml/xcp/dxaml/idl/winrt/controls/microsoft.ui.xaml.controls.controls.idl": r"enum ComboBoxSelectionChangedTrigger|^\s+Committed,",
    "dxaml/test/native/external/controls/combobox/ComboBoxIntegrationTests.cpp":
        r"Validate the default value for SelectionChangedTrigger|CanRaiseTextSubmittedEventComboBox(?:Closed|Opened)",
}
GALLERY = {
    "WinUIGallery/Samples/ListView/ListViewPage.xaml": r"ListviewSelectionSupport|ListviewScrollIntoView",
    "WinUIGallery/Samples/ComboBox/ComboBoxPage.xaml": r"ComboBox(?:Inline|ItemsSource|Editable)|PlaceholderText|SelectedIndex|IsEditable",
    "WinUIGallery/Samples/ComboBox/ComboBoxPage.xaml.cs":
        r"FontSizes|Combo3_Loaded|Combo3\.SelectedIndex|Combo3_TextSubmitted|newValue < 100",
    "WinUIGallery/Samples/ScrollViewer/ScrollViewerPage.xaml":
        r"ScrollBarVisibility",
}


def main():
    args = source_parser(__doc__).parse_args()
    winui = report(args.winui_source, WINUI)
    winui["resources"] = {
        LISTBOX: resource_report(args.winui_source, LISTBOX, re.compile(r"ListBox(?:Background|Border|BorderThemeThickness)")),
        COMBOBOX: resource_report(args.winui_source, COMBOBOX, re.compile(r"ComboBox(?:ThemeMinWidth|MinHeight|PopupThemeMinWidth|PopupMaxNumberOfItems|PopupMaxNumberOfItemsThatCanBeShownOnOneSide|DropdownBorderThickness)")),
    }
    gallery = report(args.gallery_source, GALLERY)
    gallery["listbox_mentions"] = [
        str(path.relative_to(args.gallery_source))
        for path in sorted((args.gallery_source / "WinUIGallery/Samples").rglob("*.xaml"))
        if find_lines(args.gallery_source, path.relative_to(args.gallery_source), r"<(?:\w+:)?ListBox\b")
    ]
    print(json.dumps({"winui": winui, "gallery": gallery}, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
