use super::{RefreshInterval, StatusBarItem, StatusBarItemUI};

struct CurrentNode {
    name: String,
}

impl StatusBarItemUI for CurrentNode {
    fn update(&self) {
        todo!()
    }

    fn render(&self) {
        todo!()
    }

    fn get_refresh_interval(&self) -> RefreshInterval {
        return RefreshInterval::TreeUpdate;
    }
    fn init(&self) {
        todo!()
    }
}
