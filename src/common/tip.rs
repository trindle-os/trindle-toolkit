//! The little name that appears when you rest on a button.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Labels {
    #[default]
    Brief,
    WhilePointing,
    Always,
}

impl Labels {
    pub fn chosen() -> Labels {
        crate::host::host().labels()
    }
}
