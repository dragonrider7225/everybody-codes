use std::{
    fmt::{self, Debug, Display, Formatter},
    str::FromStr,
};

use dependencies::{
    nom::{bytes::complete as bytes, character::complete as character, sequence, IResult, Parser},
    nom_supreme::ParserExt,
    pooled_string::PooledString,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Node {
    id: u32,
    plug: Socket,
    left_socket: Socket,
    left_child: Option<Box<Self>>,
    right_socket: Socket,
    right_child: Option<Box<Self>>,
    data: (),
}

impl Node {
    pub fn nom_parse(s: &str) -> IResult<&str, Self> {
        sequence::tuple((
            bytes::tag("id=").precedes(character::u32),
            bytes::tag(", plug=").precedes(Socket::nom_parse),
            bytes::tag(", leftSocket=").precedes(Socket::nom_parse),
            bytes::tag(", rightSocket=").precedes(Socket::nom_parse),
            bytes::tag(", data=").precedes(character::not_line_ending),
        ))
        .map(|(id, plug, left_socket, right_socket, _data)| Self {
            id,
            plug,
            left_socket,
            left_child: None,
            right_socket,
            right_child: None,
            data: (),
        })
        .parse(s)
    }

    /// Try to insert `n` as a descendant of `self`. `n` will only be inserted as a child of a node
    /// if the socket that its plug is being inserted into matches both shape and color.
    /// If weak bonds should also be allowed, use [`insert_weak()`] instead.
    pub fn insert(&mut self, mut n: Self) -> Result<(), Self> {
        match &mut self.left_child {
            None => {
                if self.left_socket.strong_bond(&n.plug) {
                    self.left_child = Some(Box::new(n));
                    return Ok(());
                }
            }
            Some(left_child) => {
                n = match left_child.insert(n) {
                    Ok(()) => return Ok(()),
                    Err(n) => n,
                };
            }
        }
        match &mut self.right_child {
            None => {
                if self.right_socket.strong_bond(&n.plug) {
                    self.right_child = Some(Box::new(n));
                    return Ok(());
                }
            }
            Some(right_child) => {
                n = match right_child.insert(n) {
                    Ok(()) => return Ok(()),
                    Err(n) => n,
                };
            }
        }
        Err(n)
    }

    /// Try to insert `n` as a descendant of `self`. `n` will be inserted as a child at the first
    /// socket that matches either its plug's color *or* its plug's shape.
    pub fn insert_weak(&mut self, mut n: Self) -> Result<(), Self> {
        match &mut self.left_child {
            None => {
                if self.left_socket.weak_bond(&n.plug) {
                    self.left_child = Some(Box::new(n));
                    return Ok(());
                }
            }
            Some(left_child) => {
                n = match left_child.insert_weak(n) {
                    Ok(()) => return Ok(()),
                    Err(n) => n,
                };
            }
        }
        match &mut self.right_child {
            None => {
                if self.right_socket.weak_bond(&n.plug) {
                    self.right_child = Some(Box::new(n));
                    return Ok(());
                }
            }
            Some(right_child) => {
                n = match right_child.insert_weak(n) {
                    Ok(()) => return Ok(()),
                    Err(n) => n,
                };
            }
        }
        Err(n)
    }

    pub fn insert_with_displacement(&mut self, mut n: Box<Self>) -> Result<(), Box<Self>> {
        match &mut self.left_child {
            None => {
                if self.left_socket.weak_bond(&n.plug) {
                    self.left_child = Some(n);
                    return Ok(());
                }
            }
            Some(left_child) => {
                n = if self.left_socket.strong_bond(&n.plug)
                    && !self.left_socket.strong_bond(&left_child.plug)
                {
                    std::mem::replace(left_child, n)
                } else {
                    match left_child.insert_with_displacement(n) {
                        Ok(()) => return Ok(()),
                        Err(n) => n,
                    }
                };
            }
        }
        match &mut self.right_child {
            None => {
                if self.right_socket.weak_bond(&n.plug) {
                    self.right_child = Some(n);
                    return Ok(());
                }
            }
            Some(right_child) => {
                n = if self.right_socket.strong_bond(&n.plug)
                    && !self.right_socket.strong_bond(&right_child.plug)
                {
                    std::mem::replace(right_child, n)
                } else {
                    match right_child.insert_with_displacement(n) {
                        Ok(()) => return Ok(()),
                        Err(n) => n,
                    }
                };
            }
        }
        Err(n)
    }

    pub fn iter_inorder(&self) -> IterInorder<'_> {
        IterInorder {
            steps: vec![IterStep::Subtree(self)],
        }
    }
}

impl FromStr for Node {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::nom_parse
            .complete()
            .all_consuming()
            .parse(s)
            .map(|(_, ret)| ret)
            .map_err(|e| format!("{e:?}"))
    }
}

#[derive(Clone, Debug)]
enum IterStep<'s> {
    Id(u32),
    Subtree(&'s Node),
}

#[derive(Clone, Debug)]
pub struct IterInorder<'s> {
    steps: Vec<IterStep<'s>>,
}

impl Iterator for IterInorder<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.steps.pop()? {
            IterStep::Id(ret) => Some(ret),
            IterStep::Subtree(sub) => {
                if let Some(right_child) = sub.right_child.as_deref() {
                    self.steps.push(IterStep::Subtree(right_child));
                }
                self.steps.push(IterStep::Id(sub.id));
                if let Some(left_child) = sub.left_child.as_deref() {
                    self.steps.push(IterStep::Subtree(left_child));
                }
                self.next()
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct Socket {
    color: PooledString,
    shape: PooledString,
}

impl Socket {
    pub fn nom_parse(s: &str) -> IResult<&str, Self> {
        character::alpha1::<&str, _>
            .terminated(bytes::tag(" "))
            .and(character::alpha1)
            .map(|(color, shape)| Self {
                color: color.parse().unwrap(),
                shape: shape.parse().unwrap(),
            })
            .parse(s)
    }

    /// A strong bond between a socket and a plug matches both color and shape.
    pub fn strong_bond(&self, other: &Self) -> bool {
        self == other
    }

    /// A weak bond between a socket and a plug matches either color or shape.
    pub fn weak_bond(&self, other: &Self) -> bool {
        self.color == other.color || self.shape == other.shape
    }
}

impl Debug for Socket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Socket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.color, self.shape)
    }
}
