use swayipc::reply::{Event, Node, NodeLayout, NodeType, WindowChange};
use swayipc::{Connection, EventType};

fn switch_splitting(conn: &mut Connection, focused_node: Node) -> Result<(), std::io::Error> {
    // get info from parent node which unfortunately requires us to call get_tree
    let tree = conn.get_tree().unwrap();
    let parent = tree
        .find_focused_as_ref(|n| n.nodes.iter().any(|n| n.focused))
        .ok_or("No parent")
        .unwrap();
    let is_stacked = parent.layout == NodeLayout::Stacked;
    let is_tabbed = parent.layout == NodeLayout::Tabbed;

    // get info from the focused child node
    let is_floating = focused_node.node_type == NodeType::FloatingCon;
    let is_full_screen = focused_node.percent.unwrap_or(1.0) > 1.0;

    if !is_floating && !is_full_screen && !is_stacked && !is_tabbed {
        let new_layout = if focused_node.rect.height > focused_node.rect.width {
            NodeLayout::SplitV
        } else {
            NodeLayout::SplitH
        };

        if new_layout != parent.layout {
            let cmd = match new_layout {
                NodeLayout::SplitV => "splitv",
                NodeLayout::SplitH => "splith",
                _ => "nop",
            };
            conn.run_command(cmd).unwrap();
        };
    };

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let mut conn = Connection::new().unwrap();
    for event in Connection::new()
        .unwrap()
        .subscribe(&[EventType::Window])
        .unwrap()
    {
        match event.unwrap() {
            Event::Window(e) => match e.change {
                WindowChange::Focus => {
                    if let Err(err) = switch_splitting(&mut conn, e.container) {
                        println!("err: {}", err);
                    }
                }
                _ => {}
            },
            _ => unreachable!(),
        }
    }
    Ok(())
}
