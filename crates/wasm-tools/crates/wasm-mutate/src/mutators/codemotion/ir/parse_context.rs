use crate::{Error, Result};
use std::{ops::Range, fmt::Write};
use wasmparser::BlockType;

#[derive(Debug, Default)]
pub struct Ast {
    // Index of the root node
    root: usize,
    // Nodes of the tree
    nodes: Vec<Node>,
    // indexes of ifs nodes
    ifs: Vec<usize>,
    // indexeds of loop nodes
    loops: Vec<usize>,
}

impl Ast {
    /// Returns true if the Ast has if-else nodes
    pub fn has_if(&self) -> bool {
        !self.ifs.is_empty()
    }

    /// Returns the node indexes corresponding to if-else nodes
    pub fn get_ifs(&self) -> &[usize] {
        &self.ifs
    }

    /// Returns the node indexes corresponding to if-else nodes
    pub fn get_loops(&self) -> &[usize] {
        &self.loops
    }

    /// Returns the `Root` node index of the Ast
    pub fn get_root(&self) -> usize {
        self.root
    }

    /// Returns all Ast nodes
    pub fn get_nodes(&self) -> Vec<Node> {
        self.nodes.clone()
    }

    /// Pretty print AST tree
    pub fn pretty(&self) -> String {

        let mut astrep = String::new();
        let mut worklist = vec![(self.root, 1)];

        while let Some((r, ident)) = worklist.pop() {

            for _ in 0..ident {
                astrep.write_str("\t");
            }
            let parent = &self.nodes[r];

            match parent {
                Node::IfElse { consequent, alternative, ty, range } => {
                    astrep.write_str(&format!("If({r})(ty:{:?}) {}:{}\n", ty, range.start, range.end));
                    for _ in 0..ident + 1 {
                        astrep.write_str("\t");
                    }
                    astrep.write_str(&format!("then\n"));

                    for &i in consequent {
                        worklist.push((i, ident + 1));
                    }
                    if let Some(alternative) = alternative {
                     
                        for _ in 0..ident + 1 {
                            astrep.write_str("\t");
                        }
                        astrep.write_str(&format!("else\n"));
                        for &i in alternative {
                            worklist.push((i, ident + 1));
                        }
                    }
                },
                Node::Code { range } => {
                    astrep.write_str(&format!("Code({r}) {}:{}", range.start, range.end));
                 },
                Node::Loop { body, ty, range } => {

                    astrep.write_str(&format!("Loop({r})(ty:{:?}) {}:{}", ty, range.start, range.end));
                    for &i in body {
                        worklist.push((i, ident + 1));
                    }
                },
                Node::Block { body, ty, range } => {
                    astrep.write_str(&format!("Block({r})(ty:{:?}) {}:{}", ty, range.start, range.end));
                    for &i in body {
                        worklist.push((i, ident + 1));
                    }
                },
                Node::Root(body) => {
                    for &i in body {
                        worklist.push((i, ident));
                    }
                },
            }
        }

        astrep
    }

}

#[derive(Debug, Clone)]
pub enum Node {
    /// `if`/`else` node.
    IfElse {
        /// Then branch node indices.
        consequent: Vec<usize>,
        /// Else branch node indices.
        alternative: Option<Vec<usize>>,
        /// The block type for the branches.
        ty: BlockType,
        range: Range<usize>,
    },
    /// Code node
    Code {
        /// Range on the instructions stream
        range: Range<usize>,
    },
    /// Loop Node
    Loop {
        /// Children nodes
        body: Vec<usize>,
        /// Block type
        ty: BlockType,
        /// Range on the instructions stream
        range: Range<usize>,
    },
    /// Block Node
    Block {
        /// Children nodes
        body: Vec<usize>,
        /// Block type
        ty: BlockType,
        /// Range on the instructions stream
        range: Range<usize>,
    },
    /// Special node to wrap the root nodes of the Ast
    Root(Vec<usize>),
}

#[derive(Debug)]
pub(crate) enum State {
    If,
    Else,
    Loop,
    Block,
    Root,
}

#[derive(Debug)]
pub(crate) struct ParseContext {
    current_parsing: Vec<usize>,
    stack: Vec<Vec<usize>>,
    frames: Vec<(State, Option<BlockType>, usize)>,
    current_code_range: Range<usize>,
    nodes: Vec<Node>,

    ifs: Vec<usize>,
    loops: Vec<usize>,
    blocks: Vec<usize>,
}

impl Default for ParseContext {
    fn default() -> Self {
        ParseContext {
            current_code_range: 0..0,
            current_parsing: Vec::new(),
            stack: Vec::new(),
            frames: Vec::new(),
            nodes: Vec::new(),
            ifs: Vec::new(),
            loops: Vec::new(),
            blocks: Vec::new(),
        }
    }
}

impl ParseContext {
    /// Saves current parsed nodes in the stack
    pub fn push_state(&mut self) {
        self.stack.push(self.current_parsing.clone());
        self.current_parsing = vec![]
    }

    /// Pops nodes previously parsed, set them as the current parsing
    /// and then returns a copy of the poped value
    pub fn pop_state(&mut self) -> Result<Vec<usize>> {
        match self.stack.pop() {
            Some(new_state) => {
                self.current_parsing = new_state.clone();
                Ok(new_state)
            }
            None => Err(Error::other("`pop_state` on an empty stack")),
        }
    }

    /// Push a node to the current parsing
    pub fn push_node_to_current_parsing(&mut self, node: Node) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node.clone());
        self.current_parsing.push(id);

        match node {
            Node::IfElse {
                consequent: _,
                alternative: _,
                ty: _,
                range: _,
            } => self.ifs.push(id),
            Node::Loop { .. } => self.loops.push(id),
            Node::Block { .. } => self.blocks.push(id),
            _ => {}
        }
        id
    }

    /// Push a new frame,
    ///
    /// * `state` - `If`, `Else`, `Block`, or `Loop` frame type
    /// * `ty` - Returning type of the frame
    /// * `idx` - Instruction index
    pub fn push_frame(&mut self, state: State, ty: Option<BlockType>, idx: usize) {
        self.frames.push((state, ty, idx))
    }

    /// Pop frame from the current parsing
    pub fn pop_frame(&mut self) -> Result<(State, Option<BlockType>, usize)> {
        match self.frames.pop() {
            Some(e) => Ok(e),
            None => Err(Error::other("`pop_frame` on an empty frame stack")),
        }
    }

    /// Returns the nodes indexes already parsed in the current state
    pub fn get_current_parsing(&self) -> Vec<usize> {
        self.current_parsing.clone()
    }

    /// Checks if the corrent code parsing has at least one instruction
    pub fn current_code_is_empty(&self) -> bool {
        self.current_code_range.start == self.current_code_range.end
    }

    /// Pushes the current code parsing as a `Node::Code` instance
    pub fn push_current_code_as_node(&mut self) -> usize {
        self.push_node_to_current_parsing(Node::Code {
            range: self.current_code_range.clone(),
        })
    }

    /// Resets the current code parsing
    pub fn reset_code_range_at(&mut self, idx: usize) {
        self.current_code_range = idx..idx;
    }

    /// Augmnents current code parsing to include the next instruction
    /// in the Wasm code
    pub fn append_instruction_to_current_code(&mut self) {
        self.current_code_range.end += 1;
    }

    /// Closes the parsing, creates a `Node::Root` node where the children are the current
    /// parsed nodes in the current state.
    ///
    /// Returns a new Ast instance
    pub fn finish(mut self) -> Ast {
        let roots = self.get_current_parsing();
        let root_id = self.push_node_to_current_parsing(Node::Root(roots));
        Ast {
            root: root_id,
            nodes: self.nodes,
            ifs: self.ifs,
            loops: self.loops,
        }
    }
}
