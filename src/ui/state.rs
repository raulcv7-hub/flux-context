use crate::core::file::FileNode;
use ratatui::widgets::ListState;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Represents a node in the UI tree.
#[derive(Debug, Clone)]
pub struct UiNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub expanded: bool,
    pub selected: bool,
    pub depth: usize,
    pub children: Vec<usize>,
}

/// The application state logic.
pub struct App {
    pub nodes: Vec<UiNode>,
    pub root_indices: Vec<usize>,
    pub list_state: ListState,
    pub view_items: Vec<usize>,
    pub should_quit: bool,
    pub confirmed: bool,
}

impl App {
    /// Initialize the App from the scanned file list.
    pub fn new(files: &[FileNode], _root_path: &Path) -> Self {
        let mut nodes = Vec::new();
        let mut path_to_index: HashMap<PathBuf, usize> = HashMap::new();
        let mut root_indices = Vec::new();

        for file in files {
            let relative = &file.relative_path;
            let mut current_path = PathBuf::new();
            let mut parent_idx: Option<usize> = None;

            for component in relative.components() {
                current_path.push(component);

                if !path_to_index.contains_key(&current_path) {
                    let is_dir = current_path != *relative;
                    let name = component.as_os_str().to_string_lossy().to_string();
                    let depth = current_path.components().count() - 1;

                    let node = UiNode {
                        path: current_path.clone(),
                        name,
                        is_dir,
                        expanded: true,
                        selected: true,
                        depth,
                        children: Vec::new(),
                    };

                    let idx = nodes.len();
                    nodes.push(node);
                    path_to_index.insert(current_path.clone(), idx);

                    if let Some(p) = parent_idx {
                        nodes[p].children.push(idx);
                        nodes[p].is_dir = true;
                    } else {
                        root_indices.push(idx);
                    }
                }
                parent_idx = Some(path_to_index[&current_path]);
            }
        }

        let mut app = Self {
            nodes,
            root_indices,
            list_state: ListState::default(),
            view_items: Vec::new(),
            should_quit: false,
            confirmed: false,
        };

        app.update_view();
        if !app.view_items.is_empty() {
            app.list_state.select(Some(0));
        }

        app
    }

    /// Rebuilds the view_items vector based on expansion state.
    pub fn update_view(&mut self) {
        self.view_items.clear();
        let roots = self.root_indices.clone();
        for root_idx in roots {
            self.collect_visible(root_idx);
        }
    }

    fn collect_visible(&mut self, idx: usize) {
        self.view_items.push(idx);

        let (is_dir, expanded, children) = {
            let node = &self.nodes[idx];
            (node.is_dir, node.expanded, node.children.clone())
        };

        if is_dir && expanded {
            for child_idx in children {
                self.collect_visible(child_idx);
            }
        }
    }

    pub fn toggle_selection(&mut self) {
        if let Some(selected_idx) = self.list_state.selected() {
            if let Some(&node_idx) = self.view_items.get(selected_idx) {
                let new_state = !self.nodes[node_idx].selected;
                self.set_recursive_selection(node_idx, new_state);
            }
        }
    }

    fn set_recursive_selection(&mut self, idx: usize, state: bool) {
        self.nodes[idx].selected = state;
        let children = self.nodes[idx].children.clone();
        for child_idx in children {
            self.set_recursive_selection(child_idx, state);
        }
    }

    pub fn toggle_expand(&mut self) {
        if let Some(selected_idx) = self.list_state.selected() {
            if let Some(&node_idx) = self.view_items.get(selected_idx) {
                if self.nodes[node_idx].is_dir {
                    self.nodes[node_idx].expanded = !self.nodes[node_idx].expanded;
                    self.update_view();
                }
            }
        }
    }

    pub fn move_up(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn move_down(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.view_items.len() - 1 {
                    self.view_items.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn confirm(&mut self) {
        self.confirmed = true;
        self.should_quit = true;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn get_selected_paths(&self) -> HashSet<PathBuf> {
        self.nodes
            .iter()
            .filter(|n| n.selected && !n.is_dir)
            .map(|n| n.path.clone())
            .collect()
    }
}
