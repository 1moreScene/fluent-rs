# Fluent System Icons

The 20px and 12px Regular SVGs are from Microsoft Fluent System Icons
[1.1.346](https://github.com/microsoft/fluentui-system-icons/tree/67e6dec1ee956af3ab4c746a2b3bb6a2e5520a44/assets).
Their artwork is unchanged and is redistributed under the accompanying MIT
license. The project uses the SVG files through the tintable Slint
`FluentSystemIcon` primitive.

`person-20-regular.svg` and `people-20-regular.svg` use the same pinned revision's
`assets/Person/SVG/ic_fluent_person_20_regular.svg` and
`assets/People/SVG/ic_fluent_people_20_regular.svg`. PersonPicture uses these
licensed vectors in place of the platform-specific Segoe symbol font; their
contours are not claimed to match E77B/E716 exactly.

`settings-animated.svg` contains the four winding paths from the MIT-licensed
Microsoft WinUI `AnimatedSettingsVisualSource.cpp` at commit
`07e0354d2e06572501ef48078a3cc91138749c9f`. Its view box preserves the geometry's
center and scale from the 48×48 animated source. The accompanying MIT license
also applies to this asset.
