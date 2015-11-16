use super::node::{ANode, NodeInputSlot, YNode};

/// Sends a partial stream from the output of a YNode to the (left) input of
///   another YNode.
pub struct YYSend {
	src: YNode,
	dest: YNode,
}

/// Sends an automation stream from the output of a ANode to the (right) input
/// of a YNode
pub struct AYSend {
	src: ANode,
	dest: YNode,
}

/// Sends an automation stream from the output of an ANode to the input of
/// another ANode (either the left or right slot, as specified)
pub struct AASend {
	src: ANode,
	dest: ANode,
	dest_slot: NodeInputSlot,
}

pub enum Send {
	AASend(AASend),
	AYSend(AYSend),
	YYSend(YYSend),
}