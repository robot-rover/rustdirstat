use crate::parse_tree::Dir;

use super::treeview::TreeWalk;

impl<'a> TreeWalk for &'a Dir {
    const N_COLS: usize = 3;

    fn children(&self) -> impl Iterator<Item = Self> {
        self.get_dirs().iter()
    }

    fn to_cols(&self) -> Vec<String> {
        vec![
            self.get_name().to_string(),
            self.get_size().total_size.to_string(),
            self.get_size().files_size.to_string(),
        ]
    }
}