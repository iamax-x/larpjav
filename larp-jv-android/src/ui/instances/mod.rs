//! Instances module

pub mod editor;
pub mod version_selector;

use crate::ui::instances::editor::InstanceEditorUi;
use crate::ui::instances::version_selector::VersionSelectorUi;

pub struct InstancesUi {
    editor: InstanceEditorUi,
    version_selector: VersionSelectorUi,
}

impl InstancesUi {
    pub fn new(instance_manager: std::sync::Arc<crate::instances::InstanceManager>) -> Self {
        Self {
            editor: InstanceEditorUi::new(instance_manager),
            version_selector: VersionSelectorUi::new(),
        }
    }
}