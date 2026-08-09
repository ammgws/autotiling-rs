use std::io;

use clap::Parser;
use swayipc::{Connection, Event, EventType, Node, NodeLayout, NodeType, WindowChange};

fn find_focused_workspace(node: &Node) -> Option<&Node> {
    if node.node_type == NodeType::Workspace
        && node.find_focused_as_ref(|n| n.focused).is_some()
    {
        return Some(node);
    }

    for child in &node.nodes {
        if let Some(workspace) = find_focused_workspace(child) {
            return Some(workspace);
        }
    }

    None
}

/// Returns the number of split levels between the focused container
/// and its workspace.
///
/// This follows the same logic as the original autotiling implementation:
/// only containers with more than one child count toward the depth.
///
/// For example:
///
/// workspace
/// ├── master
/// └── stack
///     ├── window
///     └── window
///
/// has depth 2 for the bottom windows.
fn focused_depth(node: &Node, target_id: i64) -> Option<u32> {
    fn walk(
        node: &Node,
        target_id: i64,
        depth: u32,
    ) -> Option<u32> {
        if node.id == target_id {
            return Some(depth);
        }

        for child in &node.nodes {
            let child_depth = if node.nodes.len() > 1 {
                depth + 1
            } else {
                depth
            };

            if let Some(found) = walk(child, target_id, child_depth) {
                return Some(found);
            }
        }

        None
    }

    walk(node, target_id, 0)
}

fn switch_splitting(
    conn: &mut Connection,
    workspaces: &[i32],
    depth_limit: u32,
) -> Result<(), String> {
    // We cannot use the event's container because its geometry can be stale.
    // Always get a fresh tree.
    //
    // https://github.com/swaywm/sway/issues/5873
    let tree = conn.get_tree().map_err(|_| "get_tree() failed")?;

    let focused_node = tree
        .find_focused_as_ref(|n| n.focused)
        .ok_or("Could not find the focused node")?;

    let workspace = find_focused_workspace(&tree)
        .ok_or("Could not find focused workspace")?;

    // Check if the focused workspace is in the allowed list.
    // Empty list means all workspaces are allowed.
    if !workspaces.is_empty() {
        let workspace_num = workspace
            .num
            .ok_or("Focused workspace has no number")?;

        if !workspaces.contains(&workspace_num) {
            return Ok(());
        }
    }

    // Floating containers should not be autotiled.
    if focused_node.node_type == NodeType::FloatingCon {
        return Ok(());
    }

    // A focused container with no parent cannot be split.
    let parent = find_parent(&tree, focused_node.id);

    let Some(parent) = parent else {
        return Ok(());
    };

    // Ignore fullscreen containers.
    if focused_node.percent.unwrap_or(1.0) > 1.0 {
        return Ok(());
    }

    // Ignore stacked and tabbed layouts.
    if parent.layout == NodeLayout::Stacked
        || parent.layout == NodeLayout::Tabbed
    {
        return Ok(());
    }

    if depth_limit > 0 {
        let depth = focused_depth(workspace, focused_node.id)
            .ok_or("Could not determine focused container depth")?;

        if depth >= depth_limit {
            return Ok(());
        }
    }

    // Determine which orientation this focused window should have.
    //
    // Tall/narrow window -> vertical split
    // Wide window        -> horizontal split
    let new_layout = if focused_node.rect.height
        > focused_node.rect.width
    {
        NodeLayout::SplitV
    } else {
        NodeLayout::SplitH
    };

    // Nothing to do if the parent already has the correct layout.
    if new_layout == parent.layout {
        return Ok(());
    }

    let command = match new_layout {
        NodeLayout::SplitV => "splitv",
        NodeLayout::SplitH => "splith",
        _ => return Ok(()),
    };

    conn.run_command(command)
        .map_err(|_| format!("{} failed", command))?;

    Ok(())
}

/// Find the direct parent of a node by walking the tree.
///
/// We return a reference into `tree`, so the lifetime is tied directly
/// to the tree passed to this function.
fn find_parent(node: &Node, target_id: i64) -> Option<&Node> {
    for child in &node.nodes {
        if child.id == target_id {
            return Some(node);
        }

        if let Some(parent) = find_parent(child, target_id) {
            return Some(parent);
        }
    }

    None
}

#[derive(Parser)]
#[clap(version, author, about)]
struct Cli {
    /// Activate autotiling only on this workspace.
    /// More than one workspace may be specified.
    #[clap(long, short = 'w', value_delimiter = ' ', num_args = 1..)]
    workspace: Vec<i32>,

    /// Limit autotiling depth.
    ///
    /// 0 = unlimited.
    /// 2 = master/stack style layout.
    #[clap(long, short = 'l', default_value = "0")]
    limit: u32,
}

fn main() -> Result<(), io::Error> {
    let args = Cli::parse();

    let mut conn = Connection::new().unwrap();

    for event in Connection::new()
        .unwrap()
        .subscribe(&[EventType::Window])
        .unwrap()
    {
        match event.unwrap() {
            Event::Window(e) => {
                if let WindowChange::Focus = e.change {
                    if let Err(err) = switch_splitting(
                        &mut conn,
                        &args.workspace,
                        args.limit,
                    ) {
                        eprintln!("err: {}", err);
                    }
                }
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}
