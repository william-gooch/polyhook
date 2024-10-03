use std::collections::{linked_list::Cursor, LinkedList};

type Id = uuid::Uuid;

#[derive(Debug)]
enum Node {
    Stitch {
        id: Id,
        ty: &'static str,
        inserts: Vec<Id>,
    },
    ChainSpace {
        id: Id,
        surrounding_nodes: Vec<Id>,
    },
}

impl Node {
    fn id(&self) -> Id {
        match self {
            Self::Stitch { id, .. } => *id,
            Self::ChainSpace { id, .. } => *id,
        }
    }

    fn chain() -> Self {
        let id = Id::new_v4();
        Self::Stitch {
            id,
            ty: "ch",
            inserts: vec![]
        }
    }

    fn dc(into: Id) -> Self {
        let id = Id::new_v4();
        Self::Stitch {
            id,
            ty: "dc",
            inserts: vec![into],
        }
    }

    fn decrease(s1: Id, s2: Id) -> Self {
        let id = Id::new_v4();
        Self::Stitch {
            id,
            ty: "dec",
            inserts: vec![s1, s2],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pattern() {
        let mut row1 = LinkedList::<Node>::new();
        // base chain
        for i in 0..=15 {
            row1.push_back(Node::chain());
        }
        row1.push_back(Node::chain());

        // dc row
        let mut row2 = LinkedList::<Node>::new();
        row1.iter().rev().skip(1).for_each(|ch| {
            row2.push_back(Node::dc(ch.id()));
        });
        row2.push_back(Node::chain());

        // decrease row
        let mut row3 = LinkedList::<Node>::new();
        {
            let mut iter = row2.iter().rev().skip(1);
            while let Some(s1) = iter.next() {
                if let Some(s2) = iter.next() {
                    row3.push_back(Node::decrease(s1.id(), s2.id()));
                } else {
                    row3.push_back(Node::dc(s1.id()));
                }
            }
        }
        row3.push_back(Node::chain());

        // decrease row
        let mut row4 = LinkedList::<Node>::new();
        {
            let mut iter = row3.iter().rev().skip(1);
            while let Some(s1) = iter.next() {
                if let Some(s2) = iter.next() {
                    row4.push_back(Node::decrease(s1.id(), s2.id()));
                } else {
                    row4.push_back(Node::dc(s1.id()));
                }
            }
        }
        row4.push_back(Node::chain());

        let pattern = row1.into_iter()
            .chain(row2)
            .chain(row3)
            .chain(row4)
            .collect::<Vec<Node>>();

        pattern.iter()
            .for_each(|s| match s {
                Node::Stitch { id, ty, inserts } => println!("{:?}: {:?} into {:?}", id, ty, inserts),
                _ => ()
            })
    }
}
