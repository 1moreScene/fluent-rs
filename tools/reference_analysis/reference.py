"""Shared read-only Git and XAML helpers for reference reports."""

import argparse
import os
from pathlib import Path
import re
import subprocess
import xml.etree.ElementTree as ET

PROJECT = Path(__file__).resolve().parents[2]
XAML = "{http://schemas.microsoft.com/winfx/2006/xaml}"


def source_parser(description):
    parser = argparse.ArgumentParser(description=description)
    for source, variable, directory in (
        ("winui", "WINUI_SOURCE_DIR", "microsoft-ui-xaml"),
        ("gallery", "WINUI_GALLERY_DIR", "WinUI-Gallery"),
    ):
        default = os.environ.get(variable, PROJECT / "references" / directory)
        parser.add_argument(f"--{source}-source", type=Path, default=Path(default))
    return parser


def tag(element):
    return element.tag.rsplit("}", 1)[-1]


def git(root, *args):
    return subprocess.check_output(
        ["git", "-C", str(root), *args], encoding="utf-8"
    ).strip()


def revision(root):
    return {
        "remote": git(root, "remote", "get-url", "origin"),
        "branch": git(root, "branch", "--show-current") or None,
        "commit": git(root, "rev-parse", "HEAD"),
    }


def read_xml(root, path):
    return ET.parse(root / path).getroot()


def find_lines(root, path, pattern):
    return [
        {"line": number, "match": match.group(0)}
        for number, line in enumerate((root / path).read_text(encoding="utf-8-sig").splitlines(), 1)
        for match in re.finditer(pattern, line)
    ]


def resource_report(root, path, keys):
    document = read_xml(root, path)
    dictionaries = [document]
    for node in document:
        if tag(node) == "ResourceDictionary.ThemeDictionaries":
            dictionaries.extend(node)

    themes = {}
    for dictionary in dictionaries:
        resources = {}
        for resource in dictionary:
            name = resource.get(XAML + "Key")
            if not name or not keys.fullmatch(name):
                continue
            resources[name] = {
                "type": tag(resource),
                "value": (resource.text or "").strip() or None,
                "attributes": {k: v for k, v in resource.attrib.items() if k != XAML + "Key"},
                "gradient": [
                    node.attrib for node in resource.iter() if tag(node) in {"GradientStop", "ScaleTransform"}
                ],
            }
        if resources:
            themes[dictionary.get(XAML + "Key", "shared")] = resources
    return themes
