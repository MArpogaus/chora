//! What XP's Explorer shows, without the drawing: the places it goes to,
//! the files of a folder in XP's order, their kinds by extension, sizes
//! in XP's words, and the history of Back, Forward and Up.

use std::path::PathBuf;

/// What an Explorer window shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    /// A folder of the file system.
    Folder(PathBuf),
    /// My Computer: the user's documents and the drives.
    Computer,
    /// The Recycle Bin.
    Trash,
}

impl Place {
    /// The title of the window and the text of the address bar: the name
    /// of a folder, or its path for `/`.
    pub fn title(&self) -> String {
        match self {
            Self::Folder(path) => path.file_name().map_or_else(
                || path.to_string_lossy().into_owned(),
                |n| n.to_string_lossy().into_owned(),
            ),
            Self::Computer => "My Computer".into(),
            Self::Trash => "Recycle Bin".into(),
        }
    }

    /// The text of the address bar: the path of a folder.
    pub fn address(&self) -> String {
        match self {
            Self::Folder(path) => path.to_string_lossy().into_owned(),
            _ => self.title(),
        }
    }

    /// The place that Up goes to: the parent of a folder, My Computer for
    /// `/`, nothing for My Computer and the Recycle Bin.
    pub fn up(&self) -> Option<Self> {
        match self {
            Self::Folder(path) => Some(
                path.parent()
                    .map_or(Self::Computer, |p| Self::Folder(p.into())),
            ),
            Self::Computer | Self::Trash => None,
        }
    }
}

/// The places of Back and Forward around the current one.
#[derive(Clone, Debug)]
pub struct History {
    back: Vec<Place>,
    forward: Vec<Place>,
    /// The place that shows.
    pub current: Place,
}

impl History {
    /// A history that starts at a place.
    pub fn new(place: Place) -> Self {
        Self {
            back: Vec::new(),
            forward: Vec::new(),
            current: place,
        }
    }

    /// Goes to a place: the current one goes back, and Forward forgets.
    pub fn go(&mut self, place: Place) {
        if place == self.current {
            return;
        }
        self.back.push(std::mem::replace(&mut self.current, place));
        self.forward.clear();
    }

    /// Whether Back has a place.
    pub fn can_back(&self) -> bool {
        !self.back.is_empty()
    }

    /// Whether Forward has a place.
    pub fn can_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    /// Goes back; false when there is no place.
    pub fn back(&mut self) -> bool {
        let Some(place) = self.back.pop() else {
            return false;
        };
        self.forward
            .push(std::mem::replace(&mut self.current, place));
        true
    }

    /// Goes forward; false when there is no place.
    pub fn forward(&mut self) -> bool {
        let Some(place) = self.forward.pop() else {
            return false;
        };
        self.back.push(std::mem::replace(&mut self.current, place));
        true
    }

    /// Goes up, as a new place; false at the top.
    pub fn up(&mut self) -> bool {
        let Some(place) = self.current.up() else {
            return false;
        };
        self.go(place);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_forward_and_up_walk_the_history() {
        let folder = |p: &str| Place::Folder(p.into());
        let mut h = History::new(folder("/home/u"));
        assert!(!h.can_back() && !h.back());
        h.go(folder("/home/u/Music"));
        assert!(h.up());
        assert_eq!(h.current, folder("/home/u"));
        assert!(h.back());
        assert_eq!(h.current, folder("/home/u/Music"));
        assert!(h.back() && h.can_forward());
    }

    #[test]
    fn forward_and_up_walk_the_history() {
        let folder = |p: &str| Place::Folder(p.into());
        let mut h = History::new(folder("/home/u"));
        h.go(folder("/home/u/Music"));
        h.back();
        assert_eq!(h.current, folder("/home/u"));
        assert!(h.forward());
        assert_eq!(h.current, folder("/home/u/Music"));
        // A new place forgets Forward.
        h.back();
        h.go(Place::Computer);
        assert!(!h.can_forward());
        assert!(!h.up());
        assert_eq!(folder("/").up(), Some(Place::Computer));
        assert_eq!(folder("/").title(), "/");
        assert_eq!(folder("/home/u").title(), "u");
    }
}
