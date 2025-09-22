use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
};

use dependencies::{
    nom::{
        self, branch, bytes::complete as bytes, character::complete as character, sequence,
        IResult, Parser,
    },
    nom_supreme,
};

#[derive(Clone, Copy, Debug)]
struct Entry<Rank, Info> {
    rank: Rank,
    info: Info,
}

impl<Rank, Info> Display for Entry<Rank, Info>
where
    Rank: Display,
    Info: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{},{}]", self.rank, self.info)
    }
}

impl<Rank, Info> PartialEq for Entry<Rank, Info>
where
    Rank: PartialOrd,
{
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(std::cmp::Ordering::Equal)
    }
}

impl<Rank, Info> Eq for Entry<Rank, Info> where Rank: PartialOrd {}

impl<Rank, Info> PartialOrd for Entry<Rank, Info>
where
    Rank: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.rank.partial_cmp(&other.rank)
    }
}

impl<Rank, Info> Ord for Entry<Rank, Info>
where
    Rank: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank.cmp(&other.rank)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Node<T> {
    value: T,
    left: TreeNode<T>,
    right: TreeNode<T>,
}

impl<T> Node<T> {
    fn new(value: T) -> Box<Self> {
        Box::new(Self {
            value,
            left: TreeNode::Empty,
            right: TreeNode::Empty,
        })
    }
}

impl<T> Node<T>
where
    T: Ord,
{
    /// Inserts the value as a descendent of this node and returns the number of nodes between this
    /// node and the newly-inserted node.
    fn insert(&mut self, value: T) -> usize {
        use std::cmp::Ordering;

        let subtree = match self.value.cmp(&value) {
            Ordering::Equal => panic!("Reused value"),
            Ordering::Less => &mut self.right,
            Ordering::Greater => &mut self.left,
        };
        if let Some(subtree) = subtree.as_deref_mut() {
            subtree.insert(value) + 1
        } else {
            *subtree = TreeNode::Normal(Self::new(value));
            0
        }
    }
}

impl<T> Node<T> {
    /// Removes and returns the first proper subtree of this node for which `cmp(subtree.value)`
    /// returns `Ordering::Equal`. This function exhausts the left subtree first unless
    /// `cmp(value)` returns `Ordering::Greater`, in which case the right subtree is exhausted
    /// first.
    fn trim_by(
        &mut self,
        mut cmp: impl FnMut(&T) -> std::cmp::Ordering,
    ) -> Option<(Box<Self>, GapId)> {
        use std::cmp::Ordering;

        let mut future = vec![];
        match cmp(&self.value) {
            Ordering::Less | Ordering::Equal => {
                future.push(&mut self.right);
                future.push(&mut self.left);
            }
            Ordering::Greater => {
                future.push(&mut self.left);
                future.push(&mut self.right);
            }
        }
        while let Some(next) = future.pop() {
            if next.is_empty() {
                continue;
            }
            match cmp(&next.as_deref().unwrap().value) {
                Ordering::Equal => return next.take_normal(),
                Ordering::Greater => {
                    let next_inner = next.as_deref_mut().unwrap();
                    future.push(&mut next_inner.left);
                    future.push(&mut next_inner.right);
                }
                Ordering::Less => {
                    let next_inner = next.as_deref_mut().unwrap();
                    future.push(&mut next_inner.right);
                    future.push(&mut next_inner.left);
                }
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GapId(usize);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum TreeNode<T> {
    #[default]
    Empty,
    Normal(Box<Node<T>>),
    Stolen(GapId),
}

impl<T> TreeNode<T> {
    pub fn take(&mut self) -> (Self, GapId) {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        let id = GapId(COUNTER.fetch_add(1, Ordering::Relaxed));
        let ret = std::mem::take(self);
        *self = Self::Stolen(id);
        (ret, id)
    }

    pub fn take_normal(&mut self) -> Option<(Box<Node<T>>, GapId)> {
        match self.take() {
            (Self::Normal(node), n) => Some((node, n)),
            (this @ (Self::Empty | Self::Stolen(_)), _) => {
                *self = this;
                None
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty | Self::Stolen(_) => true,
            Self::Normal(_) => false,
        }
    }

    pub fn as_deref(&self) -> Option<&Node<T>> {
        match self {
            Self::Normal(n) => Some(&**n),
            Self::Empty | Self::Stolen(_) => None,
        }
    }

    pub fn as_deref_mut(&mut self) -> Option<&mut Node<T>> {
        match self {
            Self::Normal(node) => Some(&mut **node),
            Self::Empty | Self::Stolen(_) => None,
        }
    }

    /// Try to replace a `Stolen(gap_id)` node in this subtree with `node`. If no such node exists,
    /// return `Some(node)` to try again elsewhere.
    fn untrim(&mut self, node: Box<Node<T>>, gap_id: GapId) -> Option<Box<Node<T>>> {
        match self {
            Self::Stolen(id) if id == &gap_id => {
                *self = Self::Normal(node);
                None
            }
            Self::Empty | Self::Stolen(_) => Some(node),
            Self::Normal(n) => {
                let node = n.left.untrim(node, gap_id)?;
                n.right.untrim(node, gap_id)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Tree<Rank, Info> {
    root: TreeNode<Entry<Rank, Info>>,
    widths: Vec<usize>,
}

impl<Rank, Info> Tree<Rank, Info> {
    fn new() -> Self {
        Self {
            root: TreeNode::Empty,
            widths: vec![],
        }
    }

    fn recalculate_widths(&mut self) {
        let mut future = vec![];
        if let Some(root) = self.root.as_deref() {
            future.push((0, root));
        }
        self.widths.clear();
        while let Some((depth, node)) = future.pop() {
            if depth == self.widths.len() {
                self.widths.push(0);
            }
            self.widths[depth] += 1;
            node.right
                .as_deref()
                .inspect(|&right| future.push((depth + 1, right)));
            node.left
                .as_deref()
                .inspect(|&left| future.push((depth + 1, left)));
        }
    }

    fn widest_row(&self) -> WidestRow<'_, Rank, Info> {
        WidestRow::new(self)
    }
}

impl<Rank, Info> Tree<Rank, Info>
where
    Rank: Ord,
{
    fn insert(&mut self, entry: Entry<Rank, Info>) {
        let depth = if let Some(root) = self.root.as_deref_mut() {
            root.insert(entry) + 1
        } else {
            self.root = TreeNode::Normal(Node::new(entry));
            0
        };
        if depth == self.widths.len() {
            self.widths.push(0);
        }
        self.widths[depth] += 1;
    }

    fn get_by_rank(&mut self, rank: &Rank) -> Option<&mut Node<Entry<Rank, Info>>> {
        use std::cmp::Ordering;

        let mut future = vec![self.root.as_deref_mut()?];
        loop {
            let next = future.pop()?;
            match rank.cmp(&next.value.rank) {
                Ordering::Equal => return Some(next),
                Ordering::Greater => {
                    if let Some(left) = next.left.as_deref_mut() {
                        future.push(left);
                    }
                    if let Some(right) = next.right.as_deref_mut() {
                        future.push(right);
                    }
                }
                Ordering::Less => {
                    if let Some(right) = next.right.as_deref_mut() {
                        future.push(right);
                    }
                    if let Some(left) = next.left.as_deref_mut() {
                        future.push(left);
                    }
                }
            }
        }
    }

    #[expect(clippy::type_complexity)]
    fn trim_by_rank(&mut self, rank: &Rank) -> Option<(Box<Node<Entry<Rank, Info>>>, GapId)> {
        use std::cmp::Ordering;

        if self.root.is_empty() {
            None
        } else if rank.cmp(&self.root.as_deref().unwrap().value.rank) == Ordering::Equal {
            self.root.take_normal()
        } else {
            self.root
                .as_deref_mut()
                .unwrap()
                .trim_by(|value| rank.cmp(&value.rank))
        }
    }

    fn untrim(
        &mut self,
        node: Box<Node<Entry<Rank, Info>>>,
        gap_id: GapId,
    ) -> Option<Box<Node<Entry<Rank, Info>>>> {
        self.root.untrim(node, gap_id)
    }
}

impl<Rank, Info> Default for Tree<Rank, Info> {
    fn default() -> Self {
        Self::new()
    }
}

struct WidestRow<'back, Rank, Info> {
    depth: usize,
    future: Vec<(usize, &'back Node<Entry<Rank, Info>>)>,
}

impl<'back, Rank, Info> WidestRow<'back, Rank, Info> {
    fn new(tree: &'back Tree<Rank, Info>) -> Self {
        let depth = tree
            .widths
            .iter()
            .copied()
            .enumerate()
            .max_by_key(|&(idx, width)| (width, std::cmp::Reverse(idx)))
            .unwrap_or((0, 0))
            .0;
        let mut future = vec![];
        if let Some(node) = tree.root.as_deref() {
            future.push((0, node));
        }
        Self { depth, future }
    }
}

impl<'back, Rank, Info> Iterator for WidestRow<'back, Rank, Info> {
    type Item = &'back Info;

    fn next(&mut self) -> Option<Self::Item> {
        let (depth, node) = self.future.pop()?;
        if depth < self.depth {
            if let Some(right) = node.right.as_deref() {
                self.future.push((depth + 1, right));
            }
            if let Some(left) = node.left.as_deref() {
                self.future.push((depth + 1, left));
            }
            self.next()
        } else {
            Some(&node.value.info)
        }
    }
}

enum Instruction {
    Add {
        id: usize,
        left: Entry<u32, char>,
        right: Entry<u32, char>,
    },
    Swap {
        id: usize,
    },
}

impl Instruction {
    fn nom_parse(s: &str) -> IResult<&str, Self> {
        use nom::Parser;

        fn parse_add(s: &str) -> IResult<&str, Instruction> {
            use nom::Parser;
            use nom_supreme::ParserExt;

            fn parse_entry(s: &str) -> IResult<&str, Entry<u32, char>> {
                sequence::separated_pair(character::u32, bytes::tag(","), character::anychar)
                    .preceded_by(bytes::tag("["))
                    .terminated(bytes::tag("]"))
                    .map(|(rank, info)| Entry { rank, info })
                    .parse(s)
            }

            sequence::tuple((
                character::u32
                    .map(|n| n as usize)
                    .preceded_by(bytes::tag("ADD id="))
                    .terminated(bytes::tag(" ")),
                parse_entry
                    .preceded_by(bytes::tag("left="))
                    .terminated(bytes::tag(" ")),
                parse_entry.preceded_by(bytes::tag("right=")),
            ))
            .map(|(id, left, right)| Instruction::Add { id, left, right })
            .parse(s)
        }

        fn parse_swap(s: &str) -> IResult<&str, Instruction> {
            use nom_supreme::ParserExt;

            character::u32
                .map(|n| n as usize)
                .preceded_by(bytes::tag("SWAP "))
                .map(|id| Instruction::Swap { id })
                .parse(s)
        }

        branch::alt((parse_add, parse_swap)).parse(s)
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<String> {
    let instructions = input.lines().map(|line| -> io::Result<_> {
        use nom_supreme::ParserExt;

        let line = line?;
        Instruction::nom_parse
            .complete()
            .all_consuming()
            .parse(&line)
            .map(|(_, res)| res)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    });
    let mut left_tree = Tree::default();
    let mut right_tree = Tree::default();
    let mut ranks = HashMap::new();
    for instruction in instructions {
        match instruction? {
            Instruction::Add { id, left, right } => {
                if let Some(prev) = ranks.insert(id, (left.rank, right.rank)) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Got duplicate id {id:?} with ranks {:?} and {:?}",
                            prev,
                            (left.rank, right.rank)
                        ),
                    ));
                }
                left_tree.insert(left);
                right_tree.insert(right);
            }
            Instruction::Swap { id } => {
                let Some((left_rank, right_rank)) = ranks.get_mut(&id) else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Can't swap nodes with non-existent ID {id:?}"),
                    ));
                };
                let Some(left_node) = left_tree.get_by_rank(left_rank) else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Can't swap deleted node with id {id:?}"),
                    ));
                };
                let Some(right_node) = right_tree.get_by_rank(right_rank) else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Can't swap deleted node with id {id:?}"),
                    ));
                };
                std::mem::swap(&mut left_node.value, &mut right_node.value);
                std::mem::swap(left_rank, right_rank);
            }
        }
    }
    Ok(left_tree
        .widest_row()
        .chain(right_tree.widest_row())
        .collect())
}

fn part2(input: &mut dyn BufRead) -> io::Result<String> {
    part1(input)
}

fn part3(input: &mut dyn BufRead) -> io::Result<String> {
    let instructions = input.lines().map(|line| -> io::Result<_> {
        use nom_supreme::ParserExt;

        let line = line?;
        Instruction::nom_parse
            .complete()
            .all_consuming()
            .parse(&line)
            .map(|(_, res)| res)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    });
    let mut left_tree = Tree::default();
    let mut right_tree = Tree::default();
    let mut ranks = HashMap::new();
    for instruction in instructions {
        match instruction? {
            Instruction::Add { id, left, right } => {
                if let Some(prev) = ranks.insert(id, (left.rank, right.rank)) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Got duplicate id {id:?} with ranks {:?} and {:?}",
                            prev,
                            (left.rank, right.rank)
                        ),
                    ));
                }
                left_tree.insert(left);
                right_tree.insert(right);
            }
            Instruction::Swap { id } => {
                let Some((left_rank, right_rank)) = ranks.get(&id) else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Can't swap nodes with non-existent ID {id:?}"),
                    ));
                };
                let extract_subtree = |trees: &mut [&mut Tree<u32, char>], rank: u32| {
                    trees
                        .iter_mut()
                        .filter_map(|tree| tree.trim_by_rank(&rank))
                        .next()
                        .ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("Can't swap deleted node with id {id:?}"),
                            )
                        })
                };
                let (left_node, left_gap_id) =
                    extract_subtree(&mut [&mut left_tree, &mut right_tree], *left_rank)?;
                let (right_node, right_gap_id) =
                    extract_subtree(&mut [&mut right_tree, &mut left_tree], *right_rank)?;
                // TODO: Can one node ever be a descendent of the other?
                if left_tree
                    .untrim(right_node, left_gap_id)
                    .and_then(|right_node| right_tree.untrim(right_node, left_gap_id))
                    .is_some()
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Gap {left_gap_id:?} is missing"),
                    ));
                }
                if right_tree
                    .untrim(left_node, right_gap_id)
                    .and_then(|left_node| left_tree.untrim(left_node, right_gap_id))
                    .is_some()
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Gap {right_gap_id:?} is missing"),
                    ));
                }
                left_tree.recalculate_widths();
                right_tree.recalculate_widths();
            }
        }
    }
    Ok(left_tree
        .widest_row()
        .chain(right_tree.widest_row())
        .collect())
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Echoes of Enigmatus Quest 2 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "echoes-of-enigmatus_02-1.txt"
            )?))?
        );
    }
    {
        println!("Echoes of Enigmatus Quest 2 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "echoes-of-enigmatus_02-2.txt"
            )?))?
        );
    }
    {
        println!("Echoes of Enigmatus Quest 2 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "echoes-of-enigmatus_02-3.txt"
            )?))?
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_part1() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            "ADD id=1 left=[10,A] right=[30,H]\n",
            "ADD id=2 left=[15,D] right=[25,I]\n",
            "ADD id=3 left=[12,F] right=[31,J]\n",
            "ADD id=4 left=[5,B] right=[27,L]\n",
            "ADD id=5 left=[3,C] right=[28,M]\n",
            "ADD id=6 left=[20,G] right=[32,K]\n",
            "ADD id=7 left=[4,E] right=[21,N]\n",
        );
        const TEST_DATA_2: &str = concat!(
            "ADD id=1 left=[160,E] right=[175,S]\n",
            "ADD id=2 left=[140,W] right=[224,D]\n",
            "ADD id=3 left=[122,U] right=[203,F]\n",
            "ADD id=4 left=[204,N] right=[114,G]\n",
            "ADD id=5 left=[136,V] right=[256,H]\n",
            "ADD id=6 left=[147,G] right=[192,O]\n",
            "ADD id=7 left=[232,I] right=[154,K]\n",
            "ADD id=8 left=[118,E] right=[125,Y]\n",
            "ADD id=9 left=[102,A] right=[210,D]\n",
            "ADD id=10 left=[183,Q] right=[254,E]\n",
            "ADD id=11 left=[146,E] right=[148,C]\n",
            "ADD id=12 left=[173,Y] right=[299,S]\n",
            "ADD id=13 left=[190,B] right=[277,B]\n",
            "ADD id=14 left=[124,T] right=[142,N]\n",
            "ADD id=15 left=[153,R] right=[133,M]\n",
            "ADD id=16 left=[252,D] right=[276,M]\n",
            "ADD id=17 left=[258,I] right=[245,P]\n",
            "ADD id=18 left=[117,O] right=[283,!]\n",
            "ADD id=19 left=[212,O] right=[127,R]\n",
            "ADD id=20 left=[278,A] right=[169,C]\n",
        );

        let expected = "CFGNLK";
        let actual = part1(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);

        let expected = "EVERYBODYCODES";
        let actual = part1(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            "ADD id=1 left=[10,A] right=[30,H]\n",
            "ADD id=2 left=[15,D] right=[25,I]\n",
            "ADD id=3 left=[12,F] right=[31,J]\n",
            "ADD id=4 left=[5,B] right=[27,L]\n",
            "ADD id=5 left=[3,C] right=[28,M]\n",
            "SWAP 1\n",
            "SWAP 5\n",
            "ADD id=6 left=[20,G] right=[32,K]\n",
            "ADD id=7 left=[4,E] right=[21,N]\n",
        );

        let expected = "MGFLNK";
        let actual = part2(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            "ADD id=1 left=[10,A] right=[30,H]\n",
            "ADD id=2 left=[15,D] right=[25,I]\n",
            "ADD id=3 left=[12,F] right=[31,J]\n",
            "ADD id=4 left=[5,B] right=[27,L]\n",
            "ADD id=5 left=[3,C] right=[28,M]\n",
            "SWAP 1\n",
            "SWAP 5\n",
            "ADD id=6 left=[20,G] right=[32,K]\n",
            "ADD id=7 left=[4,E] right=[21,N]\n",
            "SWAP 2\n",
        );
        const TEST_DATA_2: &str = concat!(
            "ADD id=1 left=[10,A] right=[30,H]\n",
            "ADD id=2 left=[15,D] right=[25,I]\n",
            "ADD id=3 left=[12,F] right=[31,J]\n",
            "ADD id=4 left=[5,B] right=[27,L]\n",
            "ADD id=5 left=[3,C] right=[28,M]\n",
            "SWAP 1\n",
            "SWAP 5\n",
            "ADD id=6 left=[20,G] right=[32,K]\n",
            "ADD id=7 left=[4,E] right=[21,N]\n",
            "SWAP 2\n",
            "SWAP 5\n",
        );

        let expected = "DJMGL";
        let actual = part3(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);

        let expected = "DJCGL";
        let actual = part3(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
