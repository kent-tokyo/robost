//! Windows UI Automation integration.
//!
//! Provides direct access to Win32 UI Automation (UIA) for interacting with
//! controls without image recognition. Windows-only; stubs are provided on
//! other platforms so the crate compiles cross-platform.
//!
//! # Usage
//!
//! ```yaml
//! - uia_get:
//!     by: { name: "ユーザー名" }
//!     property: value
//!     save_as: username_text
//!
//! - uia_set:
//!     by: { name: "ユーザー名" }
//!     value: "{{ username }}"
//!
//! - uia_click:
//!     window: "請求管理"
//!     by: { id: "btnLogin" }
//!
//! - uia_find:
//!     by: { class: "Edit" }
//!     save_as: edit_handle
//! ```

#[derive(Debug, thiserror::Error)]
pub enum UiaError {
    #[error("UIA element not found: {0}")]
    NotFound(String),
    #[error("UIA COM error: {0}")]
    Com(String),
    #[error("UIA not supported on this platform")]
    Unsupported,
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, UiaError>;

/// How to locate a UI Automation element.
#[derive(Debug, Clone)]
pub enum UiaSelector {
    /// Match by the element's Name property (accessibility label).
    Name(String),
    /// Match by the element's AutomationId property.
    AutomationId(String),
    /// Match by the element's ClassName property.
    ClassName(String),
    /// Match by the element's control type name: "Button", "Edit", "Window", etc.
    ControlType(String),
}

impl UiaSelector {
    pub fn from_name(s: impl Into<String>) -> Self {
        Self::Name(s.into())
    }
    pub fn from_id(s: impl Into<String>) -> Self {
        Self::AutomationId(s.into())
    }
    pub fn from_class(s: impl Into<String>) -> Self {
        Self::ClassName(s.into())
    }
    pub fn from_control_type(s: impl Into<String>) -> Self {
        Self::ControlType(s.into())
    }
}

/// Properties of a UIA element, returned by `UiaFinder::element_info` for the inspect command.
pub struct ElementInfo {
    pub name: String,
    pub automation_id: String,
    pub class_name: String,
    /// Human-readable control type name, e.g. `"Button"`, `"Edit"`, `"Window"`.
    pub control_type: String,
    /// Bounding rectangle as `(x, y, width, height)` in screen coordinates.
    pub rect: (i32, i32, i32, i32),
    pub enabled: bool,
}

/// A located UI Automation element.
pub struct UiaElement {
    #[cfg(target_os = "windows")]
    inner: windows_impl::Element,
    #[cfg(not(target_os = "windows"))]
    _phantom: (),
}

/// The UI Automation root finder.
pub struct UiaFinder {
    #[cfg(target_os = "windows")]
    inner: windows_impl::Finder,
    #[cfg(not(target_os = "windows"))]
    _phantom: (),
}

impl UiaFinder {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "windows")]
        {
            Ok(Self {
                inner: windows_impl::Finder::new()?,
            })
        }
        #[cfg(not(target_os = "windows"))]
        Err(UiaError::Unsupported)
    }

    /// Find the first element matching `selector` in the entire desktop tree.
    pub fn find(&self, selector: &UiaSelector) -> Result<UiaElement> {
        #[cfg(target_os = "windows")]
        {
            let el = self.inner.find(selector)?;
            Ok(UiaElement { inner: el })
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = selector;
            Err(UiaError::Unsupported)
        }
    }

    /// Find the first top-level Window element whose name contains `title_contains` (case-insensitive).
    pub fn find_window_element(&self, title_contains: &str) -> Result<UiaElement> {
        #[cfg(target_os = "windows")]
        {
            let el = self.inner.find_window_element(title_contains)?;
            Ok(UiaElement { inner: el })
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = title_contains;
            Err(UiaError::Unsupported)
        }
    }

    /// Find the first element matching `selector` within the subtree rooted at `root`.
    pub fn find_in(&self, root: &UiaElement, selector: &UiaSelector) -> Result<UiaElement> {
        #[cfg(target_os = "windows")]
        {
            let el = self.inner.find_in(&root.inner, selector)?;
            Ok(UiaElement { inner: el })
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (root, selector);
            Err(UiaError::Unsupported)
        }
    }

    /// Find the UIA element at the given screen coordinates.
    pub fn element_at_point(&self, x: i32, y: i32) -> Result<UiaElement> {
        #[cfg(target_os = "windows")]
        {
            let el = self.inner.element_at_point(x, y)?;
            Ok(UiaElement { inner: el })
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (x, y);
            Err(UiaError::Unsupported)
        }
    }

    /// Collect all properties of a UIA element into an `ElementInfo`.
    pub fn element_info(&self, el: &UiaElement) -> ElementInfo {
        #[cfg(target_os = "windows")]
        return self.inner.element_info(&el.inner);
        #[cfg(not(target_os = "windows"))]
        {
            let _ = el;
            ElementInfo {
                name: String::new(),
                automation_id: String::new(),
                class_name: String::new(),
                control_type: String::new(),
                rect: (0, 0, 0, 0),
                enabled: false,
            }
        }
    }

    /// Return the current mouse cursor position in screen coordinates.
    pub fn cursor_pos() -> Result<(i32, i32)> {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::Foundation::POINT;
            use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
            let mut p = POINT { x: 0, y: 0 };
            GetCursorPos(&mut p).map_err(|e| UiaError::Com(e.to_string()))?;
            Ok((p.x, p.y))
        }
        #[cfg(not(target_os = "windows"))]
        Err(UiaError::Unsupported)
    }

    /// List all descendant elements under `root`, collecting their properties.
    pub fn list_descendants(&self, root: &UiaElement) -> Result<Vec<ElementInfo>> {
        #[cfg(target_os = "windows")]
        return self.inner.list_descendants(&root.inner);
        #[cfg(not(target_os = "windows"))]
        {
            let _ = root;
            Err(UiaError::Unsupported)
        }
    }
}

impl UiaElement {
    /// Read the Name property.
    pub fn get_name(&self) -> Result<String> {
        #[cfg(target_os = "windows")]
        return self.inner.get_name();
        #[cfg(not(target_os = "windows"))]
        Err(UiaError::Unsupported)
    }

    /// Read the Value property (for edit controls, etc.).
    pub fn get_value(&self) -> Result<String> {
        #[cfg(target_os = "windows")]
        return self.inner.get_value();
        #[cfg(not(target_os = "windows"))]
        Err(UiaError::Unsupported)
    }

    /// Set the Value property.
    pub fn set_value(&self, value: &str) -> Result<()> {
        #[cfg(target_os = "windows")]
        return self.inner.set_value(value);
        #[cfg(not(target_os = "windows"))]
        {
            let _ = value;
            Err(UiaError::Unsupported)
        }
    }

    /// Invoke the element's default action (equivalent to clicking a button).
    pub fn invoke(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        return self.inner.invoke();
        #[cfg(not(target_os = "windows"))]
        Err(UiaError::Unsupported)
    }

    /// Get the bounding rectangle as (x, y, width, height).
    pub fn bounding_rect(&self) -> Result<(i32, i32, i32, i32)> {
        #[cfg(target_os = "windows")]
        return self.inner.bounding_rect();
        #[cfg(not(target_os = "windows"))]
        Err(UiaError::Unsupported)
    }

    /// Enumerate immediate children.
    pub fn children(&self) -> Result<Vec<UiaElement>> {
        #[cfg(target_os = "windows")]
        {
            let children = self.inner.children()?;
            Ok(children
                .into_iter()
                .map(|el| UiaElement { inner: el })
                .collect())
        }
        #[cfg(not(target_os = "windows"))]
        Err(UiaError::Unsupported)
    }

    /// Return whether the element is currently enabled.
    pub fn is_enabled(&self) -> Result<bool> {
        #[cfg(target_os = "windows")]
        return self.inner.is_enabled();
        #[cfg(not(target_os = "windows"))]
        Err(UiaError::Unsupported)
    }

    /// Return whether the element is off-screen (not visible).
    pub fn is_offscreen(&self) -> Result<bool> {
        #[cfg(target_os = "windows")]
        return self.inner.is_offscreen();
        #[cfg(not(target_os = "windows"))]
        Err(UiaError::Unsupported)
    }

    /// Read the ClassName property.
    pub fn get_class_name(&self) -> Result<String> {
        #[cfg(target_os = "windows")]
        return self.inner.get_class_name();
        #[cfg(not(target_os = "windows"))]
        Err(UiaError::Unsupported)
    }

    /// Select a named item inside a ComboBox or ListBox.
    ///
    /// For ComboBoxes the element is expanded first, then the child whose Name
    /// matches `item_name` is selected via `IUIAutomationSelectionItemPattern`.
    pub fn select_item(&self, item_name: &str) -> Result<()> {
        #[cfg(target_os = "windows")]
        return self.inner.select_item(item_name);
        #[cfg(not(target_os = "windows"))]
        {
            let _ = item_name;
            Err(UiaError::Unsupported)
        }
    }

    /// Set the checked state of a checkbox via `IUIAutomationTogglePattern`.
    pub fn set_checked(&self, checked: bool) -> Result<()> {
        #[cfg(target_os = "windows")]
        return self.inner.set_checked(checked);
        #[cfg(not(target_os = "windows"))]
        {
            let _ = checked;
            Err(UiaError::Unsupported)
        }
    }
}

// ── Windows implementation ─────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::{ElementInfo, UiaError, UiaSelector};
    use windows::{
        core::{Interface, BSTR},
        Win32::{
            Foundation::POINT,
            System::{
                Com::{
                    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
                },
                Variant::VARIANT,
            },
            UI::Accessibility::{
                CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationValuePattern,
                TreeScope_Children, TreeScope_Descendants, UIA_AutomationIdPropertyId,
                UIA_ClassNamePropertyId, UIA_ControlTypePropertyId, UIA_NamePropertyId,
                UIA_ValuePatternId, UIA_WindowControlTypeId,
            },
        },
    };

    fn control_type_id(name: &str) -> Option<i32> {
        match name.to_lowercase().as_str() {
            "button" => Some(50000),
            "calendar" => Some(50001),
            "checkbox" | "check_box" => Some(50002),
            "combobox" | "combo_box" => Some(50003),
            "edit" => Some(50004),
            "hyperlink" => Some(50005),
            "image" => Some(50006),
            "listitem" | "list_item" => Some(50007),
            "list" => Some(50008),
            "menu" => Some(50009),
            "menubar" | "menu_bar" => Some(50010),
            "menuitem" | "menu_item" => Some(50011),
            "progressbar" | "progress_bar" => Some(50012),
            "radiobutton" | "radio_button" => Some(50013),
            "scrollbar" | "scroll_bar" => Some(50014),
            "slider" => Some(50015),
            "spinner" => Some(50016),
            "statusbar" | "status_bar" => Some(50017),
            "tab" => Some(50018),
            "tabitem" | "tab_item" => Some(50019),
            "text" => Some(50020),
            "toolbar" | "tool_bar" => Some(50021),
            "tooltip" | "tool_tip" => Some(50022),
            "tree" => Some(50023),
            "treeitem" | "tree_item" => Some(50024),
            "custom" => Some(50025),
            "group" => Some(50026),
            "thumb" => Some(50027),
            "datagrid" | "data_grid" => Some(50028),
            "dataitem" | "data_item" => Some(50029),
            "document" => Some(50030),
            "splitbutton" | "split_button" => Some(50031),
            "window" => Some(50032),
            "pane" => Some(50033),
            "header" => Some(50034),
            "headeritem" | "header_item" => Some(50035),
            "table" => Some(50036),
            "titlebar" | "title_bar" => Some(50037),
            "separator" => Some(50038),
            _ => None,
        }
    }

    fn control_type_name(id: i32) -> &'static str {
        match id {
            50000 => "Button",
            50001 => "Calendar",
            50002 => "CheckBox",
            50003 => "ComboBox",
            50004 => "Edit",
            50005 => "Hyperlink",
            50006 => "Image",
            50007 => "ListItem",
            50008 => "List",
            50009 => "Menu",
            50010 => "MenuBar",
            50011 => "MenuItem",
            50012 => "ProgressBar",
            50013 => "RadioButton",
            50014 => "ScrollBar",
            50015 => "Slider",
            50016 => "Spinner",
            50017 => "StatusBar",
            50018 => "Tab",
            50019 => "TabItem",
            50020 => "Text",
            50021 => "ToolBar",
            50022 => "ToolTip",
            50023 => "Tree",
            50024 => "TreeItem",
            50025 => "Custom",
            50026 => "Group",
            50027 => "Thumb",
            50028 => "DataGrid",
            50029 => "DataItem",
            50030 => "Document",
            50031 => "SplitButton",
            50032 => "Window",
            50033 => "Pane",
            50034 => "Header",
            50035 => "HeaderItem",
            50036 => "Table",
            50037 => "TitleBar",
            50038 => "Separator",
            _ => "Unknown",
        }
    }

    pub struct Finder {
        automation: IUIAutomation,
    }

    pub struct Element {
        pub(crate) el: IUIAutomationElement,
        automation: IUIAutomation,
    }

    impl Finder {
        pub fn new() -> super::Result<Self> {
            unsafe {
                CoInitializeEx(None, COINIT_MULTITHREADED)
                    .ok()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let automation: IUIAutomation =
                    CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                        .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok(Self { automation })
            }
        }

        /// Find the first element matching `selector` under `root_el` (searching descendants).
        fn find_first_from(
            &self,
            root_el: &IUIAutomationElement,
            selector: &UiaSelector,
        ) -> super::Result<Element> {
            unsafe {
                enum CondValue {
                    Str(String),
                    Int(i32),
                }
                let (prop_id, cond_value) = match selector {
                    UiaSelector::Name(s) => (UIA_NamePropertyId, CondValue::Str(s.clone())),
                    UiaSelector::AutomationId(s) => {
                        (UIA_AutomationIdPropertyId, CondValue::Str(s.clone()))
                    }
                    UiaSelector::ClassName(s) => {
                        (UIA_ClassNamePropertyId, CondValue::Str(s.clone()))
                    }
                    UiaSelector::ControlType(s) => {
                        let type_id = control_type_id(s)
                            .ok_or_else(|| UiaError::Other(format!("unknown control type: {s}")))?;
                        (UIA_ControlTypePropertyId, CondValue::Int(type_id))
                    }
                };
                let variant = match cond_value {
                    CondValue::Str(s) => VARIANT::from(BSTR::from(s.as_str())),
                    CondValue::Int(i) => VARIANT::from(i),
                };
                let condition = self
                    .automation
                    .CreatePropertyCondition(prop_id, &variant)
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let el = root_el
                    .FindFirst(TreeScope_Descendants, &condition)
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok(Element {
                    el,
                    automation: self.automation.clone(),
                })
            }
        }

        pub fn find(&self, selector: &UiaSelector) -> super::Result<Element> {
            unsafe {
                let root = self
                    .automation
                    .GetRootElement()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                self.find_first_from(&root, selector)
            }
        }

        pub fn find_window_element(&self, title_contains: &str) -> super::Result<Element> {
            unsafe {
                let root = self
                    .automation
                    .GetRootElement()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let window_type_variant = VARIANT::from(UIA_WindowControlTypeId.0);
                let window_cond = self
                    .automation
                    .CreatePropertyCondition(UIA_ControlTypePropertyId, &window_type_variant)
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let el_array = root
                    .FindAll(TreeScope_Children, &window_cond)
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let count = el_array
                    .Length()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let needle = title_contains.to_lowercase();
                for i in 0..count {
                    let child = el_array
                        .GetElement(i)
                        .map_err(|e| UiaError::Com(e.to_string()))?;
                    let name = child
                        .CurrentName()
                        .unwrap_or_default()
                        .to_string()
                        .to_lowercase();
                    if name.contains(&needle) {
                        return Ok(Element {
                            el: child,
                            automation: self.automation.clone(),
                        });
                    }
                }
                Err(UiaError::NotFound(format!(
                    "window containing '{title_contains}'"
                )))
            }
        }

        pub fn find_in(&self, root: &Element, selector: &UiaSelector) -> super::Result<Element> {
            self.find_first_from(&root.el, selector)
        }

        pub fn element_at_point(&self, x: i32, y: i32) -> super::Result<Element> {
            unsafe {
                let el = self
                    .automation
                    .ElementFromPoint(POINT { x, y })
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok(Element {
                    el,
                    automation: self.automation.clone(),
                })
            }
        }

        pub fn element_info(&self, el: &Element) -> ElementInfo {
            unsafe {
                let name = el.el.CurrentName().unwrap_or_default().to_string();
                let automation_id = el.el.CurrentAutomationId().unwrap_or_default().to_string();
                let class_name = el.el.CurrentClassName().unwrap_or_default().to_string();
                let ct_id = el.el.CurrentControlType().map(|v| v.0).unwrap_or(0);
                let control_type = control_type_name(ct_id).to_string();
                let rect = el
                    .el
                    .CurrentBoundingRectangle()
                    .map(|r| (r.left, r.top, r.right - r.left, r.bottom - r.top))
                    .unwrap_or((0, 0, 0, 0));
                let enabled = el
                    .el
                    .CurrentIsEnabled()
                    .map(|b| b.as_bool())
                    .unwrap_or(false);
                ElementInfo {
                    name,
                    automation_id,
                    class_name,
                    control_type,
                    rect,
                    enabled,
                }
            }
        }

        pub fn list_descendants(&self, root: &Element) -> super::Result<Vec<ElementInfo>> {
            unsafe {
                let true_cond = self
                    .automation
                    .CreateTrueCondition()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let el_array = root
                    .el
                    .FindAll(TreeScope_Descendants, &true_cond)
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let count = el_array
                    .Length()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let mut result = Vec::with_capacity(count as usize);
                for i in 0..count {
                    let child = el_array
                        .GetElement(i)
                        .map_err(|e| UiaError::Com(e.to_string()))?;
                    let el = Element {
                        el: child,
                        automation: self.automation.clone(),
                    };
                    result.push(self.element_info(&el));
                }
                Ok(result)
            }
        }
    }

    impl Element {
        pub fn get_name(&self) -> super::Result<String> {
            unsafe {
                let bstr = self
                    .el
                    .CurrentName()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok(bstr.to_string())
            }
        }

        pub fn get_value(&self) -> super::Result<String> {
            unsafe {
                let pattern: IUIAutomationValuePattern = self
                    .el
                    .GetCurrentPattern(UIA_ValuePatternId)
                    .map_err(|e| UiaError::Com(e.to_string()))?
                    .cast()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let bstr = pattern
                    .CurrentValue()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok(bstr.to_string())
            }
        }

        pub fn set_value(&self, value: &str) -> super::Result<()> {
            unsafe {
                let pattern: IUIAutomationValuePattern = self
                    .el
                    .GetCurrentPattern(UIA_ValuePatternId)
                    .map_err(|e| UiaError::Com(e.to_string()))?
                    .cast()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                pattern
                    .SetValue(&BSTR::from(value))
                    .map_err(|e| UiaError::Com(e.to_string()))
            }
        }

        pub fn invoke(&self) -> super::Result<()> {
            use windows::Win32::UI::Accessibility::{
                IUIAutomationInvokePattern, UIA_InvokePatternId,
            };
            unsafe {
                let pattern: IUIAutomationInvokePattern = self
                    .el
                    .GetCurrentPattern(UIA_InvokePatternId)
                    .map_err(|e| UiaError::Com(e.to_string()))?
                    .cast()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                pattern.Invoke().map_err(|e| UiaError::Com(e.to_string()))
            }
        }

        pub fn bounding_rect(&self) -> super::Result<(i32, i32, i32, i32)> {
            unsafe {
                let rect = self
                    .el
                    .CurrentBoundingRectangle()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok((
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                ))
            }
        }

        pub fn children(&self) -> super::Result<Vec<Element>> {
            use windows::Win32::UI::Accessibility::TreeScope_Children;
            unsafe {
                let true_cond = self
                    .automation
                    .CreateTrueCondition()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let el_array = self
                    .el
                    .FindAll(TreeScope_Children, &true_cond)
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let count = el_array
                    .Length()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let mut result = Vec::with_capacity(count as usize);
                for i in 0..count {
                    let child = el_array
                        .GetElement(i)
                        .map_err(|e| UiaError::Com(e.to_string()))?;
                    result.push(Element {
                        el: child,
                        automation: self.automation.clone(),
                    });
                }
                Ok(result)
            }
        }

        pub fn is_enabled(&self) -> super::Result<bool> {
            unsafe {
                let b = self
                    .el
                    .CurrentIsEnabled()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok(b.as_bool())
            }
        }

        pub fn is_offscreen(&self) -> super::Result<bool> {
            unsafe {
                let b = self
                    .el
                    .CurrentIsOffscreen()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok(b.as_bool())
            }
        }

        pub fn get_class_name(&self) -> super::Result<String> {
            unsafe {
                let bstr = self
                    .el
                    .CurrentClassName()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok(bstr.to_string())
            }
        }

        pub fn select_item(&self, item_name: &str) -> super::Result<()> {
            use windows::Win32::UI::Accessibility::{
                IUIAutomationExpandCollapsePattern, IUIAutomationSelectionItemPattern,
                UIA_ExpandCollapsePatternId, UIA_SelectionItemPatternId,
            };
            unsafe {
                // Try to expand (ComboBox) — ignore error if not applicable.
                if let Ok(p) = self.el.GetCurrentPattern(UIA_ExpandCollapsePatternId) {
                    if let Ok(ecp) = p.cast::<IUIAutomationExpandCollapsePattern>() {
                        let _ = ecp.Expand();
                    }
                }
                let true_cond = self
                    .automation
                    .CreateTrueCondition()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let el_array = self
                    .el
                    .FindAll(TreeScope_Descendants, &true_cond)
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let count = el_array
                    .Length()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                for i in 0..count {
                    let child = el_array
                        .GetElement(i)
                        .map_err(|e| UiaError::Com(e.to_string()))?;
                    let name = child
                        .CurrentName()
                        .map_err(|e| UiaError::Com(e.to_string()))?;
                    if name == item_name {
                        if let Ok(p) = child.GetCurrentPattern(UIA_SelectionItemPatternId) {
                            let sip = p
                                .cast::<IUIAutomationSelectionItemPattern>()
                                .map_err(|e| UiaError::Com(e.to_string()))?;
                            sip.Select().map_err(|e| UiaError::Com(e.to_string()))?;
                            return Ok(());
                        }
                    }
                }
                Err(UiaError::NotFound(format!("item '{item_name}'")))
            }
        }

        pub fn set_checked(&self, checked: bool) -> super::Result<()> {
            use windows::Win32::UI::Accessibility::{
                IUIAutomationTogglePattern, ToggleState_On, UIA_TogglePatternId,
            };
            unsafe {
                let p = self
                    .el
                    .GetCurrentPattern(UIA_TogglePatternId)
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let tp = p
                    .cast::<IUIAutomationTogglePattern>()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let state = tp
                    .CurrentToggleState()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let is_on = state == ToggleState_On;
                if is_on != checked {
                    tp.Toggle().map_err(|e| UiaError::Com(e.to_string()))?;
                }
                Ok(())
            }
        }
    }
}
