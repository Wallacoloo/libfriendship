
/// Takes two inputs:
///   A1 (a sum of automations)
///   A2 (a sum of automations)
/// Provides one output: A1*A2, which is a sum of automations
pub struct ANode;

/// Takes two inputs:
///   Y (a sum of partials)
///   A (a sum of automations)
/// Provides one output: Y*A, which is a sum of partials
pub struct YNode;

pub enum Node {
    ANode(ANode),
    YNode(YNode),
}

/// Used primarily in Sends; Automations can either be sent to the *left* input
/// of an ANode, or to the right input.
#[derive(Clone, Copy, Debug)]
pub enum NodeInputSlot {
    Left,
    Right,
}

impl ANode {
    pub fn new() -> ANode {
        ANode
    }
}

impl YNode {
    pub fn new() -> YNode {
        YNode
    }
}
