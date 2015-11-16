/// Takes two inputs:
///   Y (a sum of partials)
///   A (a sum of automations)
/// Provides one output: Y*A, which is a sum of partials
pub struct YNode;
/// Takes two inputs:
///   A1 (a sum of automations)
///   A2 (a sum of automations)
/// Provides one output: A1*A2, which is a sum of automations
pub struct ANode;

pub enum Node {
	ANode(ANode),
	YNode(YNode),
}

/// Used primarily in Sends; Automations can either be sent to the *left* input
/// of an ANode, or to the right input.
pub enum NodeInputSlot {
	Left,
	Right,
}