"""Locate official references for selection, virtualization, overlays, and navigation."""

import json

from reference import find_lines, revision, source_parser

WINUI = {
    "controls/dev/CommonStyles/ListViewItem_themeresources.xaml":
        r'ListViewItem(?:MinHeight|Background(?:Selected)?(?:PointerOver|Pressed)?)',
    "dxaml/xcp/dxaml/lib/ListViewBase_Partial_Interaction.cpp":
        r'ListViewSelectionMode_(?:Single|Multiple|Extended)|shouldScrollIntoView',
    "dxaml/xcp/dxaml/lib/ListViewBase_Partial_DataVirtualization.cpp":
        r'different from realized UI containers',
    "dxaml/xcp/dxaml/lib/Popup_Partial.cpp":
        r'IsLightDismissEnabled',
    "dxaml/xcp/dxaml/lib/FlyoutBase_partial.cpp":
        r'g_defaultPlacementMode|GetFocusStateFromInputDeviceType',
    "controls/dev/CommonStyles/ContentDialog_themeresources.xaml":
        r'ContentDialogSmokeFill|DefaultButtonStates',
    "controls/dev/CommonStyles/Common_themeresources_any.xaml":
        r'SmokeFillColorDefault(?:Brush)?',
    "controls/dev/NavigationView/NavigationView.idl":
        r'(?:Compact|Expanded)ModeThresholdWidth',
}

GALLERY = {
    "WinUIGallery/Samples/ListView/ListViewPage.xaml":
        r'ListviewSelectionSupport|ListviewScrollIntoView|ListviewRestoreScrollPosition',
    "WinUIGallery/Samples/Popup/PopupPage.xaml":
        r'PopupOffsetPositioning|IsLightDismissEnabled',
    "WinUIGallery/Samples/Flyout/FlyoutPage.xaml":
        r'ButtonFlyout',
    "WinUIGallery/Samples/ContentDialog/ContentDialogPage.xaml":
        r'ContentDialog(?:Example|Content|ShowAsync)',
    "WinUIGallery/Samples/NavigationView/NavigationViewPage.xaml":
        r'PaneDisplayMode|CompactModeThresholdWidth',
}


def report(root, files):
    return {
        "revision": revision(root),
        "evidence": {
            path: find_lines(root, path, pattern) for path, pattern in files.items()
        },
    }


def main():
    args = source_parser(__doc__).parse_args()
    print(json.dumps({
        "winui": report(args.winui_source, WINUI),
        "gallery": report(args.gallery_source, GALLERY),
    }, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
