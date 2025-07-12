use swayipc::{Connection, EventType};
use swayipc::{Event, NodeLayout, NodeType, WindowChange};

use clap::Parser;

fn switch_splitting(conn: &mut Connection, workspaces: &[i32]) -> Result<(), String> {
    // Check if focused workspace is in "allowed list".
    // If `workspaces` is empty, skip allow all workspaces.
    if !workspaces.is_empty() {
        for workspace in conn
            .get_workspaces()
            .map_err(|_| "get_workspaces() failed")?
        {
            if workspace.focused {
                if workspaces.contains(&workspace.num) {
                    break;
                } else {
                    return Ok(());
                }
            }
        }
    }

    // get info from focused node and parent node which unfortunately requires us to call get_tree
    let tree = conn.get_tree().map_err(|_| "get_tree() failed")?;
    let focused_node = tree
        .find_focused_as_ref(|n| n.focused)
        .ok_or("Could not find the focused node")?;

    {
        // get info from the focused child node
        let is_stacked = focused_node.layout == NodeLayout::Stacked;
        let is_tabbed = focused_node.layout == NodeLayout::Tabbed;
        let is_floating = focused_node.node_type == NodeType::FloatingCon;
        let is_full_screen = focused_node.percent.unwrap_or(1.0) > 1.0;
        if is_floating || is_full_screen || is_stacked || is_tabbed {
            return Ok(());
        }
    }

    let new_layout = if focused_node.rect.height > focused_node.rect.width {
        NodeLayout::SplitV
    } else {
        NodeLayout::SplitH
    };
    let parent = tree
        .find_focused_as_ref(|n| n.nodes.iter().any(|n| n.focused))
        .ok_or("No parent")?;
    if new_layout == parent.layout {
        return Ok(());
    }
    let cmd = match new_layout {
        NodeLayout::SplitV => "splitv",
        NodeLayout::SplitH => "splith",
        _ => "nop",
    };
    conn.run_command(cmd).unwrap();
    Ok(())
}

#[derive(Parser)]
#[clap(version, author, about)]
struct Cli {
    /// Activate autotiling only on this workspace. More than one workspace may be specified.
    #[clap(long, short = 'w', value_delimiter = ' ', num_args = 1..)]
    workspace: Vec<i32>,
}

fn main() -> Result<(), std::io::Error> {
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
                    // We can not use the e.container because the data is stale.
                    // If we compare that node data with the node given from get_tree() after we
                    // delete a node we find that the e.container.rect.height and e.container.rect.width are stale,
                    // and therefore we make the wrong decision on which layout our next window should be.
                    // Refer to https://github.com/swaywm/sway/issues/5873
                    if let Err(err) = switch_splitting(&mut conn, &args.workspace) {
                        eprintln!("err: {}", err);
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
