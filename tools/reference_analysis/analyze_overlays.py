"""Locate WinUI Flyout, contextual menu, and ToolTip behavior and resources."""

import json

from analyze_composition import report
from reference import source_parser

WINUI = {
    "controls/dev/InfoBar/InfoBar_themeresources.xaml":
        r"InfoBar.*(?:Severity|MinHeight|CloseButton|Panel|Margin|FontSize)",
    "controls/dev/InfoBar/InfoBarPanel.cpp":
        r"MeasureOverride|ArrangeOverride|totalWidth > availableSize|if \(m_isVertical\)",
    "controls/dev/InfoBar/InfoBar.cpp":
        r"OnCloseButtonClick|RaiseClosingEvent|RaiseClosedEvent|OnIsOpenPropertyChanged|RaiseOpenedEvent|args->Cancel",
    "controls/dev/CommonStyles/ContentDialog_themeresources.xaml":
        r"ContentDialog(?:Min|Max|Padding|TitleMargin|ButtonSpacing|SmokeFill|TopOverlay)|VisualTransition To=\"Dialog|x:Name=\"ContentScrollViewer",
    "dxaml/xcp/dxaml/lib/ContentDialog_Partial.cpp":
        r"ProcessLayoutRootKey|GetDefaultButtonHelper|ExecuteCloseAction|m_spFocusedElementBeforeContentDialogShows|first focusable element",
    "dxaml/xcp/dxaml/lib/FlyoutBase_partial.cpp":
        r"g_defaultPlacementMode|SetPopupLightDismissBehavior|m_inputDeviceTypeUsedToOpen =|Propagate focus to the first focusable child",
    "dxaml/xcp/core/dll/focusmgr.cpp":
        r"GetFocusStateFromInputDeviceType|CFocusManager::CoerceFocusState|LastInputWasNonFocusNavigationKeyFromSIP",
    "dxaml/xcp/core/core/elements/Popup.cpp":
        r"m_savedFocusState|CloseTopmostPopup\(DirectUI::FocusState::Pointer",
    "controls/dev/CommonStyles/FlyoutPresenter_themeresources.xaml":
        r"FlyoutContentPadding|FlyoutBorderThemeThickness|AcrylicInAppFillColorDefaultBrush|<ScrollViewer\b",
    "dxaml/xcp/dxaml/themes/generic.xaml": r'<x:Double x:Key="FlyoutTheme(?:Min|Max)(?:Width|Height)">\d+',
    "dxaml/xcp/dxaml/lib/MenuFlyoutPresenter_Partial.cpp":
        r"CycleFocus|UIElement_IsFocusable|first focusable item",
    "dxaml/xcp/dxaml/lib/MenuFlyout_Partial.cpp": r"ShowAtWithOptions|SetPosition",
    "dxaml/xcp/dxaml/lib/MenuFlyoutItem_Partial.cpp": r"OnRightTappedUnhandled|Invoke\(\)|CloseSubMenu",
    "controls/dev/CommonStyles/MenuFlyout_themeresources.xaml":
        r"MenuFlyoutPresenterThemePadding|MenuFlyoutThemeMinHeight|MenuFlyoutItemThemePadding|DesktopAcrylicTransparentBrush",
    "dxaml/xcp/dxaml/lib/ToolTipService_Partial.h":
        r"(?:DEFAULT_SPI_GETMOUSEHOVERTIME|DEFAULT_SHOW_DURATION_SECONDS)\s+\d+",
    "dxaml/xcp/dxaml/lib/ToolTipService_Partial.cpp":
        r"SPI_GETMOUSEHOVERTIME|SPI_GETMESSAGEDURATION|OnOwnerGotFocus|OnOwnerLostFocus|OnOwnerPointerEntered|OnOwnerPointerExitedOrLostOrCanceled",
    "dxaml/xcp/dxaml/lib/ToolTip_Partial.cpp": r"put_IsHitTestVisible\(FALSE\)|PlacementMode_Top",
    "controls/dev/CommonStyles/ToolTip_themeresources.xaml":
        r"ToolTipContentThemeFontSize|ToolTipBorderPadding|ToolTipMaxWidth|AcrylicInAppFillColorDefaultBrush",
}

GALLERY = {
    "WinUIGallery/Samples/InfoBar/InfoBarPage.xaml":
        r"Severity=|SeverityComboBox|LongShort|IsClosable|IsIconVisible|ActionButtonComboBox",
    "WinUIGallery/Samples/ContentDialog/ContentDialogPage.xaml.cs":
        r"DefaultButton|PrimaryButtonText|SecondaryButtonText|CloseButtonText|ShowAsync",
    "WinUIGallery/Samples/Flyout/FlyoutPage.xaml": r"Button\.Flyout|SharedFlyout",
    "WinUIGallery/Samples/MenuFlyout/MenuFlyoutPage.xaml": r"<MenuFlyout(?:Item|SubItem)?\b",
    "WinUIGallery/Samples/ListView/ListViewPage.xaml": r"ListviewContextMenus|Grid\.ContextFlyout",
    "WinUIGallery/Samples/ToolTip/ToolTipPage.xaml": r"ToolTipService\.ToolTip|VerticalOffset|Placement=",
}


def main():
    args = source_parser(__doc__).parse_args()
    print(json.dumps({
        "winui": report(args.winui_source, WINUI),
        "gallery": report(args.gallery_source, GALLERY),
    }, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
