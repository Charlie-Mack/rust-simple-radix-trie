use std::fmt;
use std::time::Instant;

#[derive(Default, Clone)]
struct Node {
    children: Vec<Option<Box<Node>>>,
    value: Option<String>,
}

impl Node {
    fn new() -> Self {
        Self {
            children: vec![None; 16],
            value: None,
        }
    }

    // This function takes the reference to a node and a key and value
    // then it sets the current node to the passed in node and loops over the hex_key which is a
    // series of nibbles (for example 0x7abf would be 7, 10, 11, 15)
    // for each nibble we grow the trie by either getting the child node at the index of the nibble or inserting a new node
    // once we have the last nibble we set the value of the node to the value passed in
    fn insert(&mut self, hex_key: &str, value: String) {
        let mut cur = self;
        for nibble in hex_to_nibbles(hex_key) {
            cur = cur.children[nibble]
                .get_or_insert_with(|| Box::new(Node::new()))
                .as_mut();
        }
        cur.value = Some(value);
    }

    fn get(&self, hex_key: &str) -> Option<&String> {
        let mut cur = self;
        for nibble in hex_to_nibbles(hex_key) {
            match cur.children[nibble].as_deref() {
                Some(child) => cur = child,
                None => return None,
            }
        }
        cur.value.as_ref()
    }

    fn delete(&mut self, hex_key: &str) -> bool {
        fn delete_rec(node: &mut Node, nibbles: &[usize]) -> bool {
            if (nibbles.is_empty()) {
                node.value = None;
            } else {
                let idx = nibbles[0];
                if let Some(child) = node.children[idx].as_deref_mut() {
                    let should_prune = delete_rec(child, &nibbles[1..]);
                    if should_prune {
                        node.children[idx] = None;
                    }
                }
            }
            node.value.is_none() && node.children.iter().all(|c| c.is_none())
        }
        delete_rec(self, &hex_to_nibbles(hex_key).collect::<Vec<_>>())
    }

    fn insert_nibbles<I: IntoIterator<Item = usize>>(&mut self, nibbles: I, value: String) {
        let mut cur = self;
        for nib in nibbles {
            cur = cur.children[nib]
                .get_or_insert_with(|| Box::new(Node::new()))
                .as_mut();
        }
        cur.value = Some(value);
    }
}

// Pretty printer to visualize the trie.
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print_rec(
            f: &mut fmt::Formatter<'_>,
            node: &Node,
            prefix_path: &mut Vec<usize>,
            indent: &str,
            is_last: bool,
            is_root: bool,
        ) -> fmt::Result {
            let bullet = if is_root {
                "" // no bullet for the root line
            } else if is_last {
                "└── "
            } else {
                "├── "
            };

            // line text
            if is_root {
                // root label
                let value_str = node
                    .value
                    .as_ref()
                    .map(|v| format!(" = {}", v))
                    .unwrap_or_default();
                writeln!(f, "(root){}", value_str)?;
            } else {
                // path like "a1f"
                let path_hex: String = prefix_path
                    .iter()
                    .map(|&n| NIBBLE_TO_HEX[n] as char)
                    .collect();

                //take the last value in path_hex
                let last_hex = path_hex.chars().last().unwrap();

                let value_str = node
                    .value
                    .as_ref()
                    .map(|v| format!(" = {}", v))
                    .unwrap_or_default();
                writeln!(f, "{}{}{}{}", indent, bullet, last_hex, value_str)?;
            }

            // collect existing children in nibble order
            let mut present: Vec<(usize, &Node)> = node
                .children
                .iter()
                .enumerate()
                .filter_map(|(i, ch)| ch.as_deref().map(|c| (i, c)))
                .collect();
            present.sort_by_key(|(i, _)| *i);

            // recurse
            for (i, (nib, child)) in present.iter().enumerate() {
                let child_is_last = i + 1 == present.len();

                // extend indent: if this node isn't last, draw a vertical '│'; else just spaces
                let mut next_indent = String::from(indent);
                if !is_root {
                    next_indent.push_str(if is_last { "    " } else { "│   " });
                }

                // push nibble for path, recurse, then pop
                prefix_path.push(*nib);
                print_rec(f, child, prefix_path, &next_indent, child_is_last, false)?;
                prefix_path.pop();
            }

            Ok(())
        }

        print_rec(f, self, &mut Vec::new(), "", true, true)
    }
}
// Helpers

const NIBBLE_TO_HEX: &[u8; 16] = b"0123456789abcdef";

//This function returns an interator of nibbles from a hex
fn hex_to_nibbles(s: &str) -> impl Iterator<Item = usize> + '_ {
    s.chars().filter_map(|c| {
        let d = c.to_digit(16)?;
        Some(d as usize)
    })
}

fn main() {
    let mut trie = Node::new();

    // Insert a few keys and show the trie after each step.
    let steps = [
        ("a1f", "leaf-A1F"),
        ("a1e", "leaf-A1E"),
        ("b0", "leaf-B0"),
        ("00", "leaf-00"),
        ("af", "leaf-AF"),
    ];

    for (k, v) in steps {
        println!("=== Insert key {:>3} -> {:<12} ===", k, v);
        trie.insert(k, v.to_string());
        println!("{}", trie);
    }

    // Demonstrate get
    println!("Get a1e -> {:?}", trie.get("a1e"));
    println!("Get a1d -> {:?}", trie.get("a1d"));

    // Demonstrate delete + pruning
    println!("\n=== Delete a1f (prune if empty) ===");
    trie.delete("a1f");
    println!("{}", trie);

    println!("=== Delete a1e (prune if empty) ===");
    trie.delete("a1e");
    println!("{}", trie);

    let mut big_trie = Node::new();

    let start = Instant::now();
    for a in 0..16_i32.pow(6) {
        // convert number -> nibbles directly, no strings
        let mut nibbles = Vec::new();
        let mut x = a;
        // if x =
        while x > 0 {
            nibbles.push((x % 16) as usize);
            x /= 16;
        }
        nibbles.reverse();

        big_trie.insert_nibbles(nibbles, String::from("leaf")); // or some simple value
    }
    let duration = start.elapsed();

    println!("{}", big_trie);
    println!("Time taken: {:?}", duration);
}
