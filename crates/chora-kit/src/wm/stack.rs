//! The two stacking rules that every look shares (arch plan 1.1, D89): a
//! modal above its parent, the held window on top. The look orders the
//! rest; [`Rules::order`] takes its order, bottom first, and answers the
//! order with the two rules applied. Across looks the core orders the root
//! groups by their [`chora_scene::Order`].

/// The held window and the modals of a look, by id.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rules {
    held: Option<String>,
    modals: Vec<(String, String)>,
}

impl Rules {
    /// No held window, no modal.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The window that a person holds in a drag, or none.
    pub fn hold(&mut self, id: Option<&str>) {
        self.held = id.map(str::to_owned);
    }

    /// The held window.
    #[must_use]
    pub fn held(&self) -> Option<&str> {
        self.held.as_deref()
    }

    /// `id` is a modal of `parent`: it stays above it. A new parent
    /// replaces the old one.
    pub fn modal(&mut self, id: &str, parent: &str) {
        self.modals.retain(|(m, _)| m != id);
        self.modals.push((id.into(), parent.into()));
    }

    /// Forgets a window that went: as the held window, as a modal and as
    /// a parent.
    pub fn forget(&mut self, id: &str) {
        self.held.take_if(|h| h == id);
        self.modals.retain(|(m, p)| m != id && p != id);
    }

    /// The order of the look, bottom first, with the two rules applied:
    /// the held window goes on top, then each modal goes right above its
    /// parent where it lies below it. A cycle of modals ends after one
    /// pass per window.
    #[must_use]
    pub fn order(&self, look: &[String]) -> Vec<String> {
        let mut order = look.to_vec();
        if let Some(i) = self.held.as_ref().and_then(|h| at(&order, h)) {
            let held = order.remove(i);
            order.push(held);
        }
        for _ in 0..order.len() {
            if !self.lift_modals(&mut order) {
                break;
            }
        }
        order
    }

    /// Moves each modal that lies below its parent right above it;
    /// whether one moved.
    fn lift_modals(&self, order: &mut Vec<String>) -> bool {
        let mut moved = false;
        for (modal, parent) in &self.modals {
            let (Some(m), Some(p)) = (at(order, modal), at(order, parent)) else {
                continue;
            };
            if m < p {
                let id = order.remove(m);
                order.insert(p, id);
                moved = true;
            }
        }
        moved
    }
}

/// The place of an id in an order.
fn at(order: &[String], id: &str) -> Option<usize> {
    order.iter().position(|o| o == id)
}

#[cfg(test)]
mod tests {
    use super::Rules;

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn without_rules_the_look_orders() {
        assert_eq!(
            Rules::new().order(&ids(&["a", "b", "c"])),
            ids(&["a", "b", "c"])
        );
    }

    #[test]
    fn the_held_window_is_on_top() {
        let mut rules = Rules::new();
        rules.hold(Some("a"));
        assert_eq!(rules.held(), Some("a"));
        assert_eq!(rules.order(&ids(&["a", "b", "c"])), ids(&["b", "c", "a"]));
        rules.hold(None);
        assert_eq!(rules.order(&ids(&["a", "b", "c"])), ids(&["a", "b", "c"]));
    }

    #[test]
    fn a_modal_is_above_its_parent() {
        let mut rules = Rules::new();
        rules.modal("dialog", "editor");
        let order = rules.order(&ids(&["dialog", "term", "editor", "mail"]));
        assert_eq!(order, ids(&["term", "editor", "dialog", "mail"]));
    }

    #[test]
    fn a_modal_already_above_its_parent_stays() {
        let mut rules = Rules::new();
        rules.modal("dialog", "editor");
        let order = rules.order(&ids(&["editor", "term", "dialog"]));
        assert_eq!(order, ids(&["editor", "term", "dialog"]));
    }

    #[test]
    fn the_modal_of_the_held_window_goes_on_top_with_it() {
        let mut rules = Rules::new();
        rules.hold(Some("editor"));
        rules.modal("dialog", "editor");
        let order = rules.order(&ids(&["dialog", "editor", "term"]));
        assert_eq!(order, ids(&["term", "editor", "dialog"]));
    }

    #[test]
    fn a_held_modal_stays_above_its_parent() {
        let mut rules = Rules::new();
        rules.hold(Some("dialog"));
        rules.modal("dialog", "editor");
        let order = rules.order(&ids(&["dialog", "editor", "term"]));
        assert_eq!(order, ids(&["editor", "term", "dialog"]));
    }

    #[test]
    fn a_modal_of_a_modal_is_above_both() {
        let mut rules = Rules::new();
        rules.modal("ask", "dialog");
        rules.modal("dialog", "editor");
        let order = rules.order(&ids(&["ask", "dialog", "editor", "term"]));
        assert_eq!(order, ids(&["editor", "dialog", "ask", "term"]));
    }

    #[test]
    fn a_rule_for_a_window_the_look_does_not_show_changes_nothing() {
        let mut rules = Rules::new();
        rules.hold(Some("gone"));
        rules.modal("dialog", "elsewhere");
        let order = rules.order(&ids(&["dialog", "term"]));
        assert_eq!(order, ids(&["dialog", "term"]));
    }

    #[test]
    fn a_cycle_of_modals_ends() {
        let mut rules = Rules::new();
        rules.modal("a", "b");
        rules.modal("b", "a");
        let order = rules.order(&ids(&["a", "b", "c"]));
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn a_window_that_goes_leaves_no_rule() {
        let mut rules = Rules::new();
        rules.hold(Some("editor"));
        rules.modal("dialog", "editor");
        rules.modal("editor", "root");
        rules.forget("editor");
        assert_eq!(rules.held(), None);
        assert_eq!(rules, Rules::new());
    }

    #[test]
    fn a_new_parent_replaces_the_old_one() {
        let mut rules = Rules::new();
        rules.modal("dialog", "editor");
        rules.modal("dialog", "term");
        let order = rules.order(&ids(&["dialog", "term", "editor"]));
        assert_eq!(order, ids(&["term", "dialog", "editor"]));
    }
}
