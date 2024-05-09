use slint::{StandardListViewItem};

#[derive(Clone, Default)]
pub struct CustomListViewItem {
    pub item: StandardListViewItem,
    pub quiet: bool,
}

impl From<&str> for CustomListViewItem {
    fn from(string: &str) -> Self {
        CustomListViewItem {
            item: StandardListViewItem::from(slint::SharedString::from(string)),
            quiet: false,
        }
    }
}

impl From<slint::SharedString> for CustomListViewItem {
    fn from(string: slint::SharedString) -> Self {
        CustomListViewItem {
            item: StandardListViewItem::from(string),
            quiet: false,
        }
    }
}

impl Into<StandardListViewItem> for CustomListViewItem {
    fn into(self) -> StandardListViewItem {
        self.item
    }
}
