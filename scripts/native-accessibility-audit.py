#!/usr/bin/env python3
"""Audit a built Espanso GUI through the operating system accessibility API.

The script launches the application against a disposable copy of the canonical
accessibility fixture, then drives it only through what an assistive
technology can use: synthesized keyboard input and the platform accessibility
tree. Each backend is the API consumed by that platform's screen reader:

* Linux: AT-SPI 2 (Orca), keyboard input through ``xdotool`` on X11.
* Windows: UI Automation (Narrator), keyboard input through ``SendInput``.
* macOS: the AX API (VoiceOver), keyboard input through Quartz events.

It records every primary view in Japanese and English, walks the Tab sequence,
switches language with accessibility actions, opens a modal dialog, and writes
``report.json`` and ``report.md``. Failures exit with status 1.

This automates the tree, focus, and keyboard portion of
docs/ACCESSIBILITY_AUDIT.md. It does not listen to speech output and does not
replace the recorded human pass with Narrator, VoiceOver, and Orca.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Callable, Iterable

REPOSITORY = Path(__file__).resolve().parent.parent
FIXTURE = REPOSITORY / "tests" / "fixtures" / "accessibility"
CATALOG_SOURCE = REPOSITORY / "src" / "i18n.rs"
APP_NAME = "Espanso GUI"
LANGUAGES = ("ja", "en")
SECTIONS = (
    ("1", "Snippets"),
    ("2", "Profiles"),
    ("3", "Globals"),
    ("4", "Diagnostics"),
    ("5", "SettingsNav"),
)
# Roles that are containers or the application itself rather than controls.
CONTAINER_ROLES = {"application", "window", "group", "pane", "scroll", "dialog"}
MAX_TAB_STOPS = 90
# Pause between a held modifier and the key it modifies.
KEY_GAP = 0.1


@dataclass
class Node:
    role: str
    raw_role: str
    name: str
    focusable: bool
    enabled: bool
    focused: bool = False
    selected: bool | None = None
    value: str = ""
    in_dialog: bool = False
    depth: int = 0
    key: str = ""


@dataclass
class ViewPass:
    language: str
    view: str
    # "wide" with navigation buttons, or "compact" with one section selector.
    layout: str = ""
    nodes: list[Node] = field(default_factory=list)
    focus: list[Node] = field(default_factory=list)
    # Index of the stop that Tab returned to, or None when it never repeated.
    focus_loop_start: int | None = None


@dataclass
class Report:
    platform: str
    api: str
    app_name: str = ""
    passes: list[ViewPass] = field(default_factory=list)
    search_focus: dict[str, Node | None] = field(default_factory=dict)
    dialog_closed: dict[str, bool] = field(default_factory=dict)
    language_switched: bool = False
    findings: list[str] = field(default_factory=list)


def load_catalog(source: Path = CATALOG_SOURCE) -> dict[str, dict[str, str]]:
    """Read the typed Rust catalog so expectations follow the shipped copy."""
    text = source.read_text(encoding="utf-8")
    pattern = re.compile(
        r'(\w+)\s*=>\s*\(\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,?\s*\)',
        re.S,
    )
    unescape = lambda value: value.replace('\\"', '"').replace("\\n", "\n")  # noqa: E731
    return {
        key: {"ja": unescape(japanese), "en": unescape(english)}
        for key, japanese, english in pattern.findall(text)
    }


def normalize_name(value: object) -> str:
    return " ".join(str(value or "").split())


def section_selector(nodes: list[Node], catalog: dict[str, dict[str, str]], language: str) -> Node | None:
    """The compact layout's single section selector, when the window is too narrow for buttons."""
    name = normalize_name(catalog["Workspace"][language])
    return next((n for n in nodes if n.role == "combo_box" and n.name == name), None)


def section_is_open(
    nodes: list[Node], catalog: dict[str, dict[str, str]], view: str, language: str
) -> bool:
    label = normalize_name(catalog[view][language])
    selector = section_selector(nodes, catalog, language)
    if selector is not None:
        return selector.value.startswith(label)
    return any(n.name.startswith(label) and n.focusable and n.selected for n in nodes)


# ---------------------------------------------------------------------------
# Backends
# ---------------------------------------------------------------------------


class Backend:
    name = ""
    api = ""
    primary_modifier = "ctrl"
    reports_selection = True

    def __init__(self, pid: int):
        self.pid = pid

    def wait_for_app(self, timeout: float) -> None:
        raise NotImplementedError

    def app_name(self) -> str:
        raise NotImplementedError

    def snapshot(self) -> list[Node]:
        raise NotImplementedError

    def focused(self) -> Node | None:
        raise NotImplementedError

    def press(self, *keys: str) -> None:
        raise NotImplementedError

    def activate(self, predicate: Callable[[Node], bool]) -> bool:
        raise NotImplementedError

    def shortcut(self, key: str) -> None:
        self.press(self.primary_modifier, key)


class AtspiBackend(Backend):
    name = "linux"
    api = "AT-SPI 2"

    ROLES = {
        "application": "application",
        "frame": "window",
        "window": "window",
        "dialog": "dialog",
        "push button": "button",
        "button": "button",
        "toggle button": "button",
        "check box": "checkbox",
        "radio button": "radio",
        "entry": "text_input",
        "text": "text_input",
        "password text": "text_input",
        "combo box": "combo_box",
        "slider": "slider",
        "spin button": "spin_button",
        "heading": "heading",
        "label": "label",
        "static": "label",
        "panel": "group",
        "section": "group",
        "grouping": "group",
        "scroll pane": "scroll",
        "list": "list",
        "list item": "list_item",
        "menu item": "menu_item",
        "image": "image",
        "link": "link",
    }

    def __init__(self, pid: int):
        super().__init__(pid)
        import gi

        gi.require_version("Atspi", "2.0")
        from gi.repository import Atspi, GLib

        self.atspi = Atspi
        self.glib = GLib
        Atspi.init()
        self.app = None
        self.window_id = ""
        self.api = f"AT-SPI {'.'.join(str(part) for part in Atspi.get_version())}"

    @staticmethod
    def enable_accessibility() -> None:
        """Announce a screen reader, the way Orca does, so AccessKit publishes its tree.

        AccessKit activates on ScreenReaderEnabled, not on the general IsEnabled flag.
        """
        for flag in ("IsEnabled", "ScreenReaderEnabled"):
            subprocess.run(
                [
                    "gdbus", "call", "--session", "--dest", "org.a11y.Bus",
                    "--object-path", "/org/a11y/bus",
                    "--method", "org.freedesktop.DBus.Properties.Set",
                    "org.a11y.Status", flag, "<true>",
                ],
                check=True,
                capture_output=True,
            )

    def pump(self) -> None:
        context = self.glib.MainContext.default()
        while context.pending():
            context.iteration(False)

    def wait_for_app(self, timeout: float) -> None:
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            self.pump()
            desktop = self.atspi.get_desktop(0)
            for index in range(desktop.get_child_count()):
                app = desktop.get_child_at_index(index)
                if app is None:
                    continue
                try:
                    if app.get_process_id() == self.pid and app.get_child_count() > 0:
                        app.set_cache_mask(self.atspi.Cache.NONE)
                        self.app = app
                        break
                except Exception:
                    continue
            if self.app is not None:
                window = subprocess.run(
                    ["xdotool", "search", "--pid", str(self.pid), "--onlyvisible", "--name", APP_NAME],
                    capture_output=True,
                    text=True,
                    check=False,
                )
                if window.stdout.split():
                    self.window_id = window.stdout.split()[0]
                    return
            time.sleep(0.5)
        desktop = self.atspi.get_desktop(0)
        seen = []
        for index in range(desktop.get_child_count()):
            app = desktop.get_child_at_index(index)
            if app is not None:
                seen.append(f"{app.get_name()!r} (pid {app.get_process_id()})")
        raise TimeoutError(
            f"pid {self.pid} did not appear in the AT-SPI registry with an X11 window "
            f"(registered: {', '.join(seen) or 'none'}; window: {self.window_id or 'none'})"
        )

    def app_name(self) -> str:
        names = [normalize_name(self.app.get_name())]
        for index in range(self.app.get_child_count()):
            child = self.app.get_child_at_index(index)
            if child is not None:
                names.append(normalize_name(child.get_name()))
        return APP_NAME if APP_NAME in names else " / ".join(names)

    def to_node(self, accessible, depth: int, in_dialog: bool, key: str) -> Node:
        states = accessible.get_state_set()
        state = self.atspi.StateType
        raw_role = accessible.get_role_name() or ""
        role = self.ROLES.get(raw_role, raw_role or "unknown")
        selected = any(
            states.contains(flag) for flag in (state.CHECKED, state.PRESSED, state.SELECTED)
        )
        value = ""
        try:
            text = accessible.get_text_iface()
            if text is not None and role == "combo_box":
                value = normalize_name(text.get_text(0, text.get_character_count()))
        except Exception:
            value = ""
        return Node(
            role=role,
            raw_role=raw_role,
            value=value,
            name=normalize_name(accessible.get_name()),
            focusable=states.contains(state.FOCUSABLE),
            enabled=states.contains(state.ENABLED) or states.contains(state.SENSITIVE),
            focused=states.contains(state.FOCUSED),
            selected=selected,
            in_dialog=in_dialog or role == "dialog",
            depth=depth,
            key=key,
        )

    def walk(self) -> Iterable[tuple[object, Node]]:
        self.pump()
        stack = [(self.app, 0, False, "0")]
        while stack:
            accessible, depth, in_dialog, key = stack.pop()
            try:
                node = self.to_node(accessible, depth, in_dialog, key)
                count = accessible.get_child_count()
            except Exception:
                continue
            yield accessible, node
            children = []
            for index in range(count):
                child = accessible.get_child_at_index(index)
                if child is not None:
                    children.append((child, depth + 1, node.in_dialog, f"{key}.{index}"))
            stack.extend(reversed(children))

    def snapshot(self) -> list[Node]:
        return [node for _, node in self.walk()]

    def focused(self) -> Node | None:
        return next((node for _, node in self.walk() if node.focused and node.depth > 0), None)

    def press(self, *keys: str) -> None:
        names = {"ctrl": "Control_L", "shift": "Shift_L", "tab": "Tab", "escape": "Escape"}
        *modifiers, key = [names.get(key, key) for key in keys]
        subprocess.run(["xdotool", "windowfocus", "--sync", self.window_id], check=False)
        # A single `xdotool key ctrl+2` chord can reach the toolkit before its XKB modifier
        # state changes. Hold modifiers separately, the way a person presses a shortcut.
        for modifier in modifiers:
            subprocess.run(["xdotool", "keydown", modifier], check=True)
            time.sleep(KEY_GAP)
        subprocess.run(["xdotool", "key", key], check=True)
        for modifier in reversed(modifiers):
            time.sleep(KEY_GAP)
            subprocess.run(["xdotool", "keyup", modifier], check=True)

    def activate(self, predicate: Callable[[Node], bool]) -> bool:
        for accessible, node in self.walk():
            if predicate(node):
                action = accessible.get_action_iface()
                if action is None or action.get_n_actions() == 0:
                    return False
                return bool(action.do_action(0))
        return False


class UiaBackend(Backend):
    name = "windows"
    api = "UI Automation"

    CONTROL_TYPES = {
        50000: "button",
        50002: "checkbox",
        50003: "combo_box",
        50004: "text_input",
        50005: "link",
        50006: "image",
        50007: "list_item",
        50008: "list",
        50011: "menu_item",
        50013: "radio",
        50014: "scroll",
        50015: "slider",
        50016: "spin_button",
        50020: "label",
        50026: "group",
        50030: "text_input",
        50032: "window",
        50033: "pane",
    }
    # The title bar and system menu belong to Windows, not to the application's tree.
    SYSTEM_CHROME = {50010, 50037}
    IS_DIALOG_PROPERTY = 30174
    HEADING_LEVEL_PROPERTY = 30173
    HEADING_LEVEL_NONE = 80050

    def __init__(self, pid: int):
        super().__init__(pid)
        import ctypes
        import comtypes
        import comtypes.client

        comtypes.CoInitialize()
        comtypes.client.GetModule("UIAutomationCore.dll")
        from comtypes.gen import UIAutomationClient as lib

        self.ctypes = ctypes
        self.lib = lib
        self.uia = comtypes.client.CreateObject(lib.CUIAutomation, interface=lib.IUIAutomation)
        self.walker = self.uia.RawViewWalker
        self.window = None

    def wait_for_app(self, timeout: float) -> None:
        lib = self.lib
        condition = self.uia.CreatePropertyCondition(lib.UIA_ProcessIdPropertyId, self.pid)
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            window = self.uia.GetRootElement().FindFirst(lib.TreeScope_Children, condition)
            if window and self.walker.GetFirstChildElement(window):
                self.window = window
                return
            time.sleep(0.5)
        raise TimeoutError("the application window did not appear in UI Automation")

    def app_name(self) -> str:
        return normalize_name(self.window.CurrentName)

    def pattern(self, element, pattern_id: int, interface):
        try:
            unknown = element.GetCurrentPattern(pattern_id)
        except Exception:
            return None
        if not unknown:
            return None
        return unknown.QueryInterface(interface)

    def to_node(self, element, depth: int, in_dialog: bool) -> Node:
        lib = self.lib
        control_type = element.CurrentControlType
        localized = normalize_name(element.CurrentLocalizedControlType)
        role = self.CONTROL_TYPES.get(control_type, localized or str(control_type))
        try:
            is_dialog = bool(element.GetCurrentPropertyValue(self.IS_DIALOG_PROPERTY))
        except Exception:
            is_dialog = False
        if is_dialog or localized == "dialog":
            role = "dialog"
        try:
            heading = element.GetCurrentPropertyValue(self.HEADING_LEVEL_PROPERTY)
            if isinstance(heading, int) and heading > self.HEADING_LEVEL_NONE:
                role = "heading"
        except Exception:
            pass
        selected = None
        toggle = self.pattern(element, lib.UIA_TogglePatternId, lib.IUIAutomationTogglePattern)
        if toggle is not None:
            selected = toggle.CurrentToggleState == lib.ToggleState_On
        selection = self.pattern(
            element, lib.UIA_SelectionItemPatternId, lib.IUIAutomationSelectionItemPattern
        )
        if selection is not None:
            selected = bool(selected) or bool(selection.CurrentIsSelected)
        value = ""
        value_pattern = self.pattern(element, lib.UIA_ValuePatternId, lib.IUIAutomationValuePattern)
        if value_pattern is not None:
            value = normalize_name(value_pattern.CurrentValue)
        runtime = element.GetRuntimeId()
        return Node(
            role=role,
            raw_role=f"{control_type}:{localized}",
            value=value,
            name=normalize_name(element.CurrentName),
            focusable=bool(element.CurrentIsKeyboardFocusable),
            enabled=bool(element.CurrentIsEnabled),
            focused=bool(element.CurrentHasKeyboardFocus),
            selected=selected,
            in_dialog=in_dialog or role == "dialog",
            depth=depth,
            key=".".join(str(part) for part in runtime) if runtime else "",
        )

    def walk(self) -> Iterable[tuple[object, Node]]:
        stack = [(self.window, 0, False)]
        while stack:
            element, depth, in_dialog = stack.pop()
            try:
                node = self.to_node(element, depth, in_dialog)
            except Exception:
                continue
            yield element, node
            children = []
            child = self.walker.GetFirstChildElement(element)
            while child:
                if child.CurrentControlType not in self.SYSTEM_CHROME:
                    children.append((child, depth + 1, node.in_dialog))
                child = self.walker.GetNextSiblingElement(child)
            stack.extend(reversed(children))

    def snapshot(self) -> list[Node]:
        return [node for _, node in self.walk()]

    def focused(self) -> Node | None:
        element = self.uia.GetFocusedElement()
        if not element or element.CurrentProcessId != self.pid:
            return None
        node = self.to_node(element, 0, False)
        parent = self.walker.GetParentElement(element)
        while parent and not self.uia.CompareElements(parent, self.window):
            if self.to_node(parent, 0, False).role == "dialog":
                node.in_dialog = True
            parent = self.walker.GetParentElement(parent)
        return node

    def press(self, *keys: str) -> None:
        ctypes = self.ctypes
        user32 = ctypes.windll.user32
        codes = {"ctrl": 0x11, "shift": 0x10, "tab": 0x09, "escape": 0x1B}
        virtual = [codes[key] if key in codes else ord(key.upper()) for key in keys]
        hwnd = self.window.CurrentNativeWindowHandle
        if hwnd:
            user32.SetForegroundWindow(hwnd)
        for code in virtual:
            user32.keybd_event(code, 0, 0, 0)
            time.sleep(KEY_GAP)
        for code in reversed(virtual):
            user32.keybd_event(code, 0, 2, 0)
            time.sleep(KEY_GAP)

    def activate(self, predicate: Callable[[Node], bool]) -> bool:
        lib = self.lib
        for element, node in self.walk():
            if not predicate(node):
                continue
            invoke = self.pattern(element, lib.UIA_InvokePatternId, lib.IUIAutomationInvokePattern)
            if invoke is not None:
                invoke.Invoke()
                return True
            expand = self.pattern(
                element, lib.UIA_ExpandCollapsePatternId, lib.IUIAutomationExpandCollapsePattern
            )
            if expand is not None:
                expand.Expand()
                return True
            selection = self.pattern(
                element, lib.UIA_SelectionItemPatternId, lib.IUIAutomationSelectionItemPattern
            )
            if selection is not None:
                selection.Select()
                return True
            toggle = self.pattern(element, lib.UIA_TogglePatternId, lib.IUIAutomationTogglePattern)
            if toggle is not None:
                toggle.Toggle()
                return True
            return False
        return False


class AxBackend(Backend):
    name = "macos"
    api = "macOS Accessibility (AX)"
    primary_modifier = "cmd"

    ROLES = {
        "AXApplication": "application",
        "AXWindow": "window",
        "AXSheet": "dialog",
        "AXButton": "button",
        "AXCheckBox": "checkbox",
        "AXRadioButton": "radio",
        "AXTextField": "text_input",
        "AXTextArea": "text_input",
        "AXComboBox": "combo_box",
        "AXPopUpButton": "combo_box",
        "AXSlider": "slider",
        "AXIncrementor": "spin_button",
        "AXHeading": "heading",
        "Heading": "heading",
        "AXScrollBar": "scroll",
        "AXStaticText": "label",
        "AXGroup": "group",
        "AXScrollArea": "scroll",
        "AXList": "list",
        "AXMenuItem": "menu_item",
        "AXImage": "image",
        "AXLink": "link",
    }
    # AccessKit lets AX clients set AXFocused on every node, so settability says nothing about
    # keyboard focus. Treat enabled interactive roles as focusable instead, excluding the
    # window's own title-bar buttons.
    INTERACTIVE = {"button", "checkbox", "radio", "text_input", "combo_box", "slider", "spin_button", "link"}
    WINDOW_BUTTONS = {"AXCloseButton", "AXMinimizeButton", "AXFullScreenButton", "AXZoomButton"}
    KEY_CODES = {
        "1": 18, "2": 19, "3": 20, "4": 21, "5": 23, "f": 3,
        "tab": 48, "escape": 53, "cmd": 55, "shift": 56,
    }

    def __init__(self, pid: int):
        super().__init__(pid)
        import ApplicationServices as ax
        import Quartz
        from AppKit import NSApplicationActivateIgnoringOtherApps, NSRunningApplication

        if not ax.AXIsProcessTrusted():
            raise PermissionError(
                "this process is not trusted for the macOS Accessibility API; "
                "grant Accessibility access to the terminal or CI agent"
            )
        self.ax = ax
        self.quartz = Quartz
        self.running = NSRunningApplication.runningApplicationWithProcessIdentifier_(pid)
        self.activate_flag = NSApplicationActivateIgnoringOtherApps
        self.app = ax.AXUIElementCreateApplication(pid)

    def attribute(self, element, name: str):
        error, value = self.ax.AXUIElementCopyAttributeValue(element, name, None)
        return value if error == 0 else None

    def wait_for_app(self, timeout: float) -> None:
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            windows = self.attribute(self.app, "AXWindows") or []
            if windows and self.attribute(windows[0], "AXChildren"):
                return
            time.sleep(0.5)
        raise TimeoutError("the application window did not appear in the AX tree")

    def app_name(self) -> str:
        names = [normalize_name(self.attribute(self.app, "AXTitle"))]
        for window in self.attribute(self.app, "AXWindows") or []:
            names.append(normalize_name(self.attribute(window, "AXTitle")))
        return APP_NAME if APP_NAME in names else " / ".join(names)

    def to_node(self, element, depth: int, in_dialog: bool) -> Node:
        raw_role = str(self.attribute(element, "AXRole") or "")
        subrole = str(self.attribute(element, "AXSubrole") or "")
        description = normalize_name(self.attribute(element, "AXRoleDescription"))
        role = self.ROLES.get(raw_role, raw_role or "unknown")
        if subrole in {"AXDialog", "AXSystemDialog"} or description == "dialog":
            role = "dialog"
        if raw_role == "AXButton" and subrole == "AXToggle":
            role = "button"
        name = next(
            (
                normalize_name(value)
                for value in (
                    self.attribute(element, "AXTitle"),
                    self.attribute(element, "AXDescription"),
                    self.attribute(element, "AXLabel"),
                )
                if normalize_name(value)
            ),
            "",
        )
        if not name and role in {"label", "heading"}:
            name = normalize_name(self.attribute(element, "AXValue"))
        selected_value = self.attribute(element, "AXSelected")
        if selected_value is None and role in {"button", "checkbox", "radio"}:
            value = self.attribute(element, "AXValue")
            selected_value = bool(value) if isinstance(value, (bool, int)) else None
        enabled = self.attribute(element, "AXEnabled")
        value = self.attribute(element, "AXValue") if role == "combo_box" else None
        return Node(
            role=role,
            raw_role=f"{raw_role}/{subrole}" if subrole else raw_role,
            name=name,
            value=normalize_name(value) if isinstance(value, str) else "",
            focusable=role in self.INTERACTIVE and subrole not in self.WINDOW_BUTTONS,
            enabled=True if enabled is None else bool(enabled),
            focused=bool(self.attribute(element, "AXFocused")),
            selected=None if selected_value is None else bool(selected_value),
            in_dialog=in_dialog or role == "dialog",
            depth=depth,
            key=str(hash(element)),
        )

    def walk(self) -> Iterable[tuple[object, Node]]:
        stack = [(window, 0, False) for window in self.attribute(self.app, "AXWindows") or []]
        while stack:
            element, depth, in_dialog = stack.pop()
            try:
                node = self.to_node(element, depth, in_dialog)
            except Exception:
                continue
            yield element, node
            children = self.attribute(element, "AXChildren") or []
            stack.extend(reversed([(child, depth + 1, node.in_dialog) for child in children]))

    def snapshot(self) -> list[Node]:
        return [node for _, node in self.walk()]

    def focused(self) -> Node | None:
        element = self.attribute(self.app, "AXFocusedUIElement")
        if element is None:
            return None
        node = self.to_node(element, 0, False)
        parent = self.attribute(element, "AXParent")
        while parent is not None:
            if self.to_node(parent, 0, False).role == "dialog":
                node.in_dialog = True
            if self.attribute(parent, "AXRole") == "AXWindow":
                break
            parent = self.attribute(parent, "AXParent")
        return node

    def press(self, *keys: str) -> None:
        quartz = self.quartz
        self.running.activateWithOptions_(self.activate_flag)
        flags = 0
        if "cmd" in keys:
            flags |= quartz.kCGEventFlagMaskCommand
        if "shift" in keys:
            flags |= quartz.kCGEventFlagMaskShift
        modifiers = [self.KEY_CODES[key] for key in keys if key in {"cmd", "shift"}]
        key = next(key for key in keys if key not in {"cmd", "shift"})
        sequence = [(code, True) for code in modifiers]
        sequence += [(self.KEY_CODES[key], True), (self.KEY_CODES[key], False)]
        sequence += [(code, False) for code in reversed(modifiers)]
        for index, (code, down) in enumerate(sequence):
            event = quartz.CGEventCreateKeyboardEvent(None, code, down)
            held = modifiers if index < len(sequence) - len(modifiers) else []
            quartz.CGEventSetFlags(event, flags if held else 0)
            quartz.CGEventPost(quartz.kCGHIDEventTap, event)
            time.sleep(KEY_GAP)

    def activate(self, predicate: Callable[[Node], bool]) -> bool:
        for element, node in self.walk():
            if predicate(node):
                return self.ax.AXUIElementPerformAction(element, "AXPress") == 0
        return False


BACKENDS = {"linux": AtspiBackend, "windows": UiaBackend, "macos": AxBackend}


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------


class Driver:
    def __init__(self, backend: Backend, catalog: dict[str, dict[str, str]], delay: float):
        self.backend = backend
        self.catalog = catalog
        self.delay = delay

    def text(self, key: str, language: str) -> str:
        return normalize_name(self.catalog[key][language])

    def settle(self, factor: float = 1.0) -> None:
        time.sleep(self.delay * factor)

    def wait_until(self, predicate: Callable[[list[Node]], bool], timeout: float = 10.0) -> list[Node]:
        deadline = time.monotonic() + timeout
        nodes = self.backend.snapshot()
        while not predicate(nodes) and time.monotonic() < deadline:
            self.settle(0.5)
            nodes = self.backend.snapshot()
        return nodes

    def has_navigation(self, nodes: list[Node], language: str) -> bool:
        if section_selector(nodes, self.catalog, language) is not None:
            return True
        label = self.text("Snippets", language)
        return any(node.name.startswith(label) and node.focusable for node in nodes)

    def tab_walk(self) -> tuple[list[Node], int | None]:
        stops: list[Node] = []
        seen: dict[str, int] = {}
        for _ in range(MAX_TAB_STOPS):
            self.backend.press("tab")
            self.settle()
            focused = self.backend.focused()
            if focused is None:
                stops.append(Node("none", "", "", False, False))
                continue
            identity = f"{focused.key}|{focused.role}|{focused.name}"
            if identity in seen:
                return stops, seen[identity]
            seen[identity] = len(stops)
            stops.append(focused)
        return stops, None

    def stable_snapshot(self, nodes: list[Node], timeout: float = 10.0) -> list[Node]:
        """Wait until two consecutive snapshots agree, so lazily built rows are included."""
        deadline = time.monotonic() + timeout
        signature = [(n.role, n.name, n.focusable, n.enabled) for n in nodes]
        while time.monotonic() < deadline:
            self.settle()
            current = self.backend.snapshot()
            current_signature = [(n.role, n.name, n.focusable, n.enabled) for n in current]
            if current_signature == signature:
                return current
            nodes, signature = current, current_signature
        return nodes

    def open_section(self, shortcut: str, view: str, language: str) -> list[Node]:
        self.backend.shortcut(shortcut)
        nodes = self.wait_until(
            lambda nodes: section_is_open(nodes, self.catalog, view, language), 10
        )
        return self.stable_snapshot(nodes)

    def switch_language(self, language: str) -> str | None:
        """Switch through accessibility actions only; return what failed, or None."""
        current = "ja" if language == "en" else "en"
        self.open_section("5", "SettingsNav", current)
        label = self.text("Language", current)
        selector = lambda n: n.role == "combo_box" and n.name.startswith(label)  # noqa: E731
        self.wait_until(lambda nodes: any(selector(n) for n in nodes))
        if not self.backend.activate(selector):
            return f"the '{label}' selector exposes no usable accessibility action"
        target = "English" if language == "en" else "日本語"
        option = lambda n: n.name == target and n.role != "combo_box"  # noqa: E731
        nodes = self.wait_until(lambda nodes: any(option(n) for n in nodes))
        if not any(option(n) for n in nodes):
            return f"activating '{label}' did not expose the option '{target}'"
        self.settle()
        if not self.backend.activate(option):
            roles = sorted({n.raw_role for n in nodes if option(n)})
            return f"the option '{target}' ({', '.join(roles)}) exposes no usable accessibility action"
        # The selector's own label is translated once the new language takes effect.
        expected = self.text("Language", language)
        switched = lambda nodes: any(  # noqa: E731
            n.role == "combo_box" and n.name.startswith(expected) for n in nodes
        )
        if not switched(self.wait_until(switched)):
            return f"activating '{target}' did not change the interface language"
        return None

    def run(self, report: Report) -> None:
        self.backend.wait_for_app(90)
        self.settle(4)
        report.app_name = self.backend.app_name()
        for language in LANGUAGES:
            if language != LANGUAGES[0]:
                failure = self.switch_language(language)
                report.language_switched = failure is None
                if failure is not None:
                    report.findings.append(f"{language}: language switch failed: {failure}")
                    return
            self.wait_until(lambda nodes, language=language: self.has_navigation(nodes, language), 30)
            for shortcut, view in SECTIONS:
                nodes = self.open_section(shortcut, view, language)
                layout = "compact" if section_selector(nodes, self.catalog, language) else "wide"
                view_pass = ViewPass(language, view, layout, nodes)
                view_pass.focus, view_pass.focus_loop_start = self.tab_walk()
                report.passes.append(view_pass)
            self.open_section("1", "Snippets", language)
            self.backend.shortcut("f")
            self.settle(2)
            report.search_focus[language] = self.backend.focused()
            self.backend.press("escape")
            self.settle()
            self.audit_dialog(report, language)

    def audit_dialog(self, report: Report, language: str) -> None:
        add_file = self.text("AddFile", language)
        title = self.text("NewMatchFileTitle", language)
        view = "NewMatchFileDialog"
        is_add_file = lambda n: n.name == add_file and n.role == "button"  # noqa: E731
        if not any(is_add_file(n) for n in self.backend.snapshot()):
            # The compact layout keeps Add file inside the match-file selector.
            files = self.text("MatchFiles", language)
            self.backend.activate(lambda n: n.role == "combo_box" and n.name == files)
            self.wait_until(lambda nodes: any(is_add_file(n) for n in nodes))
        if not self.backend.activate(is_add_file):
            report.findings.append(f"{language}/{view}: '{add_file}' could not be activated")
            return
        nodes = self.stable_snapshot(
            self.wait_until(lambda nodes: any(n.role == "dialog" for n in nodes))
        )
        dialog_pass = ViewPass(language, view, "modal", nodes)
        dialog_pass.focus, dialog_pass.focus_loop_start = self.tab_walk()
        report.passes.append(dialog_pass)
        if not any(n.role == "dialog" and n.name == title for n in nodes):
            dialogs = [n.name for n in nodes if n.role == "dialog"]
            report.findings.append(
                f"{language}/{view}: expected a dialog named '{title}', found {dialogs or 'none'}"
            )
        self.backend.press("escape")
        nodes = self.wait_until(lambda nodes: not any(n.role == "dialog" for n in nodes))
        report.dialog_closed[language] = not any(n.role == "dialog" for n in nodes)


# ---------------------------------------------------------------------------
# Rules
# ---------------------------------------------------------------------------


def evaluate(report: Report, catalog: dict[str, dict[str, str]], reports_selection: bool) -> list[str]:
    findings = list(report.findings)
    text = lambda key, language: normalize_name(catalog[key][language])  # noqa: E731
    if report.app_name != APP_NAME:
        findings.append(f"application accessible name is '{report.app_name}', expected '{APP_NAME}'")

    by_view: dict[tuple[str, str], ViewPass] = {}
    for view_pass in report.passes:
        where = f"{view_pass.language}/{view_pass.view}"
        by_view[(view_pass.language, view_pass.view)] = view_pass
        controls = [
            n for n in view_pass.nodes if n.focusable and n.enabled and n.role not in CONTAINER_ROLES
        ]
        if not controls:
            findings.append(f"{where}: no focusable controls were exposed")
        for node in controls:
            if not node.name:
                findings.append(f"{where}: focusable {node.raw_role} control has no accessible name")
        named_stops = [stop for stop in view_pass.focus if stop.name]
        unnamed = [stop for stop in view_pass.focus if not stop.name and stop.role != "none"]
        for stop in unnamed:
            findings.append(f"{where}: Tab reached an unnamed {stop.raw_role} control")
        if len({(stop.role, stop.name) for stop in named_stops}) < 3:
            findings.append(f"{where}: Tab traversal reached fewer than three named controls")
        if view_pass.focus_loop_start is None:
            findings.append(f"{where}: Tab traversal did not return to its first stop within {MAX_TAB_STOPS} presses")
        elif view_pass.focus_loop_start != 0:
            stop = view_pass.focus[view_pass.focus_loop_start]
            findings.append(
                f"{where}: Tab traversal loops back to stop {view_pass.focus_loop_start + 1} "
                f"('{stop.name}') instead of the first stop, so earlier stops are not reachable again"
            )
        if view_pass.focus_loop_start is not None:
            reached = {(stop.role, stop.name) for stop in view_pass.focus}
            modal = view_pass.view.endswith("Dialog")
            for node in controls:
                if (not modal or node.in_dialog) and (node.role, node.name) not in reached:
                    findings.append(
                        f"{where}: '{node.name}' ({node.raw_role}) is exposed as focusable "
                        "but Tab never reaches it"
                    )
        if view_pass.view.endswith("Dialog"):
            escaped = [stop for stop in view_pass.focus if stop.role != "none" and not stop.in_dialog]
            for stop in escaped:
                findings.append(f"{where}: focus left the modal dialog to '{stop.name}' ({stop.raw_role})")
            continue
        selector = section_selector(view_pass.nodes, catalog, view_pass.language)
        if selector is not None:
            current = text(view_pass.view, view_pass.language)
            if not selector.value.startswith(current):
                findings.append(
                    f"{where}: section selector '{selector.name}' exposes the value "
                    f"'{selector.value}', expected the current section '{current}'"
                )
            continue
        for _, key in SECTIONS:
            label = text(key, view_pass.language)
            matches = [n for n in view_pass.nodes if n.name.startswith(label) and n.focusable]
            if not matches:
                findings.append(f"{where}: navigation control '{label}' is missing")
            elif reports_selection and key == view_pass.view and not any(n.selected for n in matches):
                findings.append(f"{where}: current navigation control '{label}' is not exposed as selected")

    for _, key in SECTIONS:
        japanese = by_view.get(("ja", key))
        english = by_view.get(("en", key))
        if japanese is None or english is None:
            continue
        counts = [
            sum(1 for n in view.nodes if n.focusable and n.enabled and n.role not in CONTAINER_ROLES)
            for view in (japanese, english)
        ]
        if counts[0] != counts[1]:
            findings.append(f"{key}: Japanese exposes {counts[0]} focusable controls but English exposes {counts[1]}")

    for language, focused in report.search_focus.items():
        expected = text("Search", language)
        if focused is None or focused.role != "text_input" or focused.name != expected:
            actual = "nothing" if focused is None else f"'{focused.name}' ({focused.raw_role})"
            findings.append(f"{language}: primary+F focused {actual}, expected search field '{expected}'")
    for language, closed in report.dialog_closed.items():
        if not closed:
            findings.append(f"{language}/NewMatchFileDialog: Escape did not close the dialog")
    missing = [language for language in LANGUAGES if language not in report.dialog_closed]
    if report.passes and missing:
        findings.append(f"dialog audit did not run for: {', '.join(missing)}")
    return findings


def write_reports(report: Report, output: Path) -> None:
    output.mkdir(parents=True, exist_ok=True)
    (output / "report.json").write_text(
        json.dumps(asdict(report), ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    lines = [
        f"# Native accessibility audit ({report.platform})",
        "",
        f"- API: {report.api}",
        f"- Application name: {report.app_name or '(not found)'}",
        f"- Language switched with accessibility actions: {'yes' if report.language_switched else 'no'}",
        f"- Result: {'FAIL' if report.findings else 'PASS'}",
        "",
        "| Language | View | Layout | Nodes | Focusable controls | Tab stops | Tab returns to |",
        "| --- | --- | --- | ---: | ---: | ---: | --- |",
    ]
    for view_pass in report.passes:
        controls = sum(
            1 for n in view_pass.nodes if n.focusable and n.enabled and n.role not in CONTAINER_ROLES
        )
        lines.append(
            f"| {view_pass.language} | {view_pass.view} | {view_pass.layout} | "
            f"{len(view_pass.nodes)} | {controls} | "
            f"{len(view_pass.focus)} | "
            + ("never" if view_pass.focus_loop_start is None else f"stop {view_pass.focus_loop_start + 1}")
            + " |"
        )
    lines += ["", "## Findings", ""]
    lines += [f"- {finding}" for finding in report.findings] or ["- None"]
    for view_pass in report.passes:
        lines += ["", f"## Tab sequence: {view_pass.language}/{view_pass.view}", ""]
        lines += [
            f"{index}. {stop.name or '(unnamed)'} — {stop.role}"
            + (" [selected]" if stop.selected else "")
            for index, stop in enumerate(view_pass.focus, 1)
        ]
    (output / "report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


# ---------------------------------------------------------------------------
# Environment
# ---------------------------------------------------------------------------


def platform_key() -> str:
    system = platform.system()
    return {"Linux": "linux", "Windows": "windows", "Darwin": "macos"}.get(system, system.lower())


def prepare_environment(key: str, sandbox: Path, allow_default_root: bool) -> tuple[dict[str, str], Path]:
    """Return the child environment and the config root the app will load."""
    environment = dict(os.environ)
    # Hide an installed Espanso so the app cannot resolve the user's real configuration.
    shim = sandbox / "bin"
    shim.mkdir(parents=True)
    if key == "windows":
        (shim / "espanso.cmd").write_text("@exit /b 1\r\n", encoding="utf-8")
    else:
        script = shim / "espanso"
        script.write_text("#!/bin/sh\nexit 1\n", encoding="utf-8")
        script.chmod(0o755)
    environment["PATH"] = f"{shim}{os.pathsep}{environment.get('PATH', '')}"
    if key == "linux":
        # xdotool delivers keys through X11, so keep the app off a native Wayland socket.
        environment.pop("WAYLAND_DISPLAY", None)
        environment["XDG_CONFIG_HOME"] = str(sandbox / "config")
        environment["XDG_DATA_HOME"] = str(sandbox / "data")
        root = sandbox / "config" / "espanso"
    elif key == "macos":
        environment["HOME"] = str(sandbox / "home")
        root = sandbox / "home" / "Library" / "Application Support" / "espanso"
    else:
        # Known-folder lookup ignores environment overrides on Windows.
        root = Path(os.environ["APPDATA"]) / "espanso"
        if not allow_default_root:
            raise SystemExit("Windows uses the real %APPDATA%\\espanso; pass --allow-default-config-root on a disposable machine")
        if root.exists():
            raise SystemExit(f"refusing to overwrite existing configuration at {root}")
    shutil.copytree(FIXTURE, root)
    return environment, root


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("binary", type=Path, help="path to the built espanso-gui executable")
    parser.add_argument("--output", type=Path, default=Path("target/accessibility-audit"))
    parser.add_argument("--delay", type=float, default=0.4, help="seconds to wait after each input")
    parser.add_argument(
        "--allow-default-config-root",
        action="store_true",
        help="allow writing the fixture into the platform's real Espanso config folder (Windows CI only)",
    )
    args = parser.parse_args(argv)

    key = platform_key()
    if key not in BACKENDS:
        raise SystemExit(f"unsupported platform: {platform.system()}")
    catalog = load_catalog()
    report = Report(platform=key, api=BACKENDS[key].api)
    with tempfile.TemporaryDirectory(prefix="espanso-gui-a11y-") as temporary:
        environment, root = prepare_environment(key, Path(temporary), args.allow_default_config_root)
        if key == "linux":
            AtspiBackend.enable_accessibility()
        log =(Path(temporary) / "app.log").open("w", encoding="utf-8")
        process = subprocess.Popen(
            [str(args.binary.resolve())], env=environment, stdout=log, stderr=subprocess.STDOUT
        )
        try:
            backend = BACKENDS[key](process.pid)
            report.api = backend.api
            Driver(backend, catalog, args.delay).run(report)
            report.findings = evaluate(report, catalog, backend.reports_selection)
        except Exception as error:  # report the failure with whatever was collected
            report.findings.append(f"audit aborted: {type(error).__name__}: {error}")
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
            log.close()
            app_log = (Path(temporary) / "app.log").read_text(encoding="utf-8", errors="replace")
            if key == "windows":
                shutil.rmtree(root, ignore_errors=True)
        write_reports(report, args.output)
        (args.output / "app.log").write_text(app_log, encoding="utf-8")
    print((args.output / "report.md").read_text(encoding="utf-8"))
    return 1 if report.findings else 0


if __name__ == "__main__":
    sys.exit(main())
