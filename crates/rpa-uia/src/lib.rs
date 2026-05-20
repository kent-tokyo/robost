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
            Ok(Self { inner: windows_impl::Finder::new()? })
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
        Err(UiaError::Unsupported)
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
        Err(UiaError::Unsupported)
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
            Ok(children.into_iter().map(|el| UiaElement { inner: el }).collect())
        }
        #[cfg(not(target_os = "windows"))]
        Err(UiaError::Unsupported)
    }
}

// ── Windows implementation ─────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::{UiaError, UiaSelector};
    use windows::{
        core::BSTR,
        Win32::{
            System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED},
            UI::Accessibility::{
                CUIAutomation, IUIAutomation, IUIAutomationCondition, IUIAutomationElement,
                IUIAutomationValuePattern, TreeScope_Descendants, UIA_AutomationIdPropertyId,
                UIA_ClassNamePropertyId, UIA_NamePropertyId, UIA_ValuePatternId,
            },
            Foundation::VARIANT,
        },
    };

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
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let automation: IUIAutomation =
                    CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                        .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok(Self { automation })
            }
        }

        pub fn find(&self, selector: &UiaSelector) -> super::Result<Element> {
            unsafe {
                let root = self.automation.GetRootElement()
                    .map_err(|e| UiaError::Com(e.to_string()))?;

                let (prop_id, value) = match selector {
                    UiaSelector::Name(s) => (UIA_NamePropertyId, s.clone()),
                    UiaSelector::AutomationId(s) => (UIA_AutomationIdPropertyId, s.clone()),
                    UiaSelector::ClassName(s) => (UIA_ClassNamePropertyId, s.clone()),
                };

                let variant = VARIANT::from(BSTR::from(value.as_str()));
                let condition = self.automation
                    .CreatePropertyCondition(prop_id, &variant)
                    .map_err(|e| UiaError::Com(e.to_string()))?;

                let el = root
                    .FindFirst(TreeScope_Descendants, &condition)
                    .map_err(|e| UiaError::Com(e.to_string()))?
                    .ok_or_else(|| UiaError::NotFound(format!("{selector:?}")))?;

                Ok(Element { el, automation: self.automation.clone() })
            }
        }
    }

    impl Element {
        pub fn get_name(&self) -> super::Result<String> {
            unsafe {
                let bstr = self.el.CurrentName()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok(bstr.to_string())
            }
        }

        pub fn get_value(&self) -> super::Result<String> {
            unsafe {
                let pattern: IUIAutomationValuePattern = self.el
                    .GetCurrentPattern(UIA_ValuePatternId)
                    .map_err(|e| UiaError::Com(e.to_string()))?
                    .cast()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let bstr = pattern.CurrentValue()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok(bstr.to_string())
            }
        }

        pub fn set_value(&self, value: &str) -> super::Result<()> {
            unsafe {
                let pattern: IUIAutomationValuePattern = self.el
                    .GetCurrentPattern(UIA_ValuePatternId)
                    .map_err(|e| UiaError::Com(e.to_string()))?
                    .cast()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                pattern.SetValue(&BSTR::from(value))
                    .map_err(|e| UiaError::Com(e.to_string()))
            }
        }

        pub fn invoke(&self) -> super::Result<()> {
            use windows::Win32::UI::Accessibility::{IUIAutomationInvokePattern, UIA_InvokePatternId};
            unsafe {
                let pattern: IUIAutomationInvokePattern = self.el
                    .GetCurrentPattern(UIA_InvokePatternId)
                    .map_err(|e| UiaError::Com(e.to_string()))?
                    .cast()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                pattern.Invoke().map_err(|e| UiaError::Com(e.to_string()))
            }
        }

        pub fn bounding_rect(&self) -> super::Result<(i32, i32, i32, i32)> {
            unsafe {
                let rect = self.el.CurrentBoundingRectangle()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                Ok((rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top))
            }
        }

        pub fn children(&self) -> super::Result<Vec<Element>> {
            use windows::Win32::UI::Accessibility::TreeScope_Children;
            unsafe {
                let true_cond = self.automation.CreateTrueCondition()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let el_array = self.el
                    .FindAll(TreeScope_Children, &true_cond)
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let count = el_array.Length()
                    .map_err(|e| UiaError::Com(e.to_string()))?;
                let mut result = Vec::with_capacity(count as usize);
                for i in 0..count {
                    let child = el_array.GetElement(i)
                        .map_err(|e| UiaError::Com(e.to_string()))?;
                    result.push(Element { el: child, automation: self.automation.clone() });
                }
                Ok(result)
            }
        }
    }
}
